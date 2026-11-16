//! Five matched, synthetic core/presentation runs. Excludes host queries/rendering.
use basalt_lab::{
    basalt_core::{Core, Frame, Point, StoneWall},
    basalt_presentation::Presenter,
};
use std::{hint::black_box, time::Instant};
fn main() {
    let mut samples = Vec::new();
    for scenario in 0..3 {
        for run in 0..5 {
            let mut frame:Frame=serde_json::from_value(serde_json::json!({"tick":1,"owner":1,"team":0,"alive":true,"pos":{"x":300000,"y":300000},"hp":2000,"max_hp":2000,"attack":100,"ult_unlocked":true,"units":[],"profile":vec![50;13],"manual":true})).unwrap();
            let mut setup = Core::default();
            if scenario > 0 {
                for id in 0..5 {
                    let at = Point::new(330000 + id * 15000, 300000);
                    setup.walls.push(StoneWall {
                        id: id as u32,
                        a: at,
                        b: at,
                        integrity: 3,
                        born: 1,
                        out_since: None,
                        slide: None,
                    });
                }
            }
            if scenario == 2 {
                setup.charging = Some((true, Point::new(700000, 200000), 1_000_000));
            }
            let mut timings = Vec::new();
            let mut states = Vec::new();
            let mut emissions = 0;
            for enabled in [false, true] {
                let mut core = setup.clone();
                let mut presenter = Presenter::default();
                let start = Instant::now();
                for t in 1..=20000 {
                    frame.tick = t;
                    let effects = core.step(&frame);
                    if enabled {
                        let v = presenter.step(&core, &frame, &effects);
                        emissions += v.events.len() + v.leases.len();
                        black_box(v);
                    }
                    black_box(&core);
                }
                timings.push(start.elapsed().as_secs_f64() * 1_000_000.0 / 20000.0);
                states.push(serde_json::to_string(&core).unwrap());
            }
            assert_eq!(states[0], states[1]);
            let label=["inactive","five_stationary_blocks","synthetic_charge_and_five_blocks"][scenario];
            samples.push(serde_json::json!({"scenario":label,"run":run+1,"core_us_per_tick":timings[0],"core_and_presentation_us_per_tick":timings[1],"emissions":emissions,"combat_state_equal":true}));
        }
    }
    println!(
        "{}",
        serde_json::json!({"samples":samples,"scope":"Five matched synthetic runs per scenario; core plus read-only projection. Excludes SDK queries, asset loading, native renderer and real-game Proceed time."})
    );
}
