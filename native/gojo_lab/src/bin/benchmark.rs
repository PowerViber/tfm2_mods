//! Release-core timings, not engine rendering/host-call timings.
use gojo_lab::gojo_core::*;
use std::{hint::black_box, time::Instant};
fn main() {
    let mut all = vec![Fighter {
        id: 1,
        team: 0,
        x: 100000,
        y: 100000,
        hp: 850,
        max_hp: 850,
        ad: 70,
        ap: 81,
        level: 3,
        champion: true,
        visible: true,
        targetable: true,
        immune: false,
        cc_immune: false,
        farmable: false,
        tower: false,
        immovable: false,
        cleansed: false,
    }];
    for id in 2..=128 {
        all.push(Fighter {
            id,
            team: if id < 5 { 0 } else { 1 },
            x: 110000 + (id as i64 % 14) * 9000,
            y: 100000 + (id as i64 / 14) * 5000,
            hp: 100000,
            max_hp: 100000,
            ad: 50,
            ap: 50,
            level: 3,
            champion: id <= 10,
            visible: true,
            targetable: true,
            immune: false,
            cc_immune: false,
            farmable: id > 10,
            tower: false,
        immovable: false,
        cleansed: false,
        });
    }
    let mut rows = vec![];
    for run in 0..5 {
        let mut c = Core::default();
        let mut times = vec![];
        let mut cold = 0;
        let mut starved = 0;
        let mut burnout = 0;
        for t in 0..20000 {
            let b = if t % 55 == 0 {
                vec![(t as u64 + 1, 5)]
            } else {
                vec![]
            };
            let i = Input {
                tick: t,
                me: &all[0],
                all: &all,
                unlocked: [true; 3],
                controlled: false,
                bound: false,
                automatic: true,
                haste: 35,
                ult_haste: 0,
                action: None,
                basics: &b,
                champion_damage: t % 60 == 0,
                shield: None,
                wall: &|_, _| false,
                barrier: &|_, _| false,
                sight: &|_, _| true,
            };
            let start = Instant::now();
            let commands = c.step(black_box(&i));
            let ns = start.elapsed().as_nanos();
            black_box(commands);
            if t == 0 {
                cold = ns
            } else {
                times.push(ns)
            }
            starved += usize::from(c.ce < 6000);
            burnout += usize::from(c.burnout > t);
        }
        times.sort_unstable();
        rows.push(format!("{{\"run\":{run},\"entities\":128,\"steps\":20000,\"coldNs\":{cold},\"medianNs\":{},\"p95Ns\":{},\"p99Ns\":{},\"requestedDamage\":{},\"flashes\":{},\"starvedTicks\":{starved},\"burnoutTicks\":{burnout}}}",times[times.len()/2],times[times.len()*95/100],times[times.len()*99/100],c.requested,c.flashes));
    }
    println!("[{}]", rows.join(","));
}
