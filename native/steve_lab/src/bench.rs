use steve_lab::{steve_craft_core::*, steve_craft_geometry::*};
fn main() {
    let mut rows = vec![];
    for count in [32, 64, 128] {
        let mut runs = vec![];
        for run in 0..5 {
            let units: Vec<_> = (0..count)
                .map(|id| Unit {
                    id,
                    team: if id == 0 || id % 3 == 0 { 0 } else { 1 },
                    x: 480000 + (id as i64 % 8) * 12000,
                    y: 480000 + (id as i64 / 8) * 12000,
                    hp: 1150,
                    max_hp: 1150,
                    ad: 62,
                    shield: 0,
                    visible: true,
                    targetable: true,
                    champion: id < 12,
                    farmable: id >= 12,
                    immune: false,
                    cc_immune: false,
                    tower: false,
                    gadget: false,
                    immovable: false,
                    epic:false,totem_locked:false,gifted_armour_until:0,
                    vx: 0,
                    vy: 0,
                    swing: false,
                })
                .collect();
            let mut core = Core::default();
            let mut timings = vec![];
            for tick in 0..1800 {
                let i = Input {
                    tick,
                    me: &units[0],
                    units: &units,
                    unlocked: [true; 3],
                    controlled: false,
                    bound: false,
                    automatic: true,
                    action: None,
                    basics: &[],
                    objects: &[],
                    wall: &|_, _| false,
                    barriers: &[],
                    reserved_returns: &[],
                home: (300000, 400000),
                    haste: 0,
                    ult_haste: 0,
                };
                let start = std::time::Instant::now();
                let out = core.step(&i);
                timings.push(start.elapsed().as_nanos());
                for cmd in out {
                    if let Cmd::Spawn { object, .. } = cmd {
                        core.allocated(object, true)
                    }
                }
            }
            timings.sort_unstable();
            let p95 = timings[timings.len() * 95 / 100];
            let p99 = timings[timings.len() * 99 / 100];
            let max = *timings.last().unwrap();
            runs.push(format!("{{\"run\":{run},\"p95Ns\":{p95},\"p99Ns\":{p99},\"maxNs\":{max},\"evaluatedBranches\":{}}}",core.metrics.branches));
        }
        rows.push(format!(
            "{{\"units\":{count},\"runs\":[{}]}}",
            runs.join(",")
        ));
    }
    // A visibility graph request is measured separately from a combat core step.
    let p = arc((480000, 480000), 0, 4);
    let seg: Vec<_> = p.windows(2).map(|p| (p[0], p[1])).collect();
    let start = std::time::Instant::now();
    for _ in 0..50 {
        std::hint::black_box(detour((480000, 520000), (480000, 650000), &seg, &|_, _| {
            false
        }));
    }
    let route = start.elapsed().as_nanos() / 50;
    println!("{{\"coreOnly\":true,\"hostAndRenderingMeasured\":false,\"stepsPerRun\":1800,\"curvedRouteMeanNs\":{route},\"crowds\":[{}]}}",rows.join(","));
}
