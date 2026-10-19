// Generated from retained pre-PR40 animation metadata. Units: 1/1000 tick.
pub fn frame(mode:&str,part:&str,elapsed:usize,expand:usize)->usize {
    let durations:&[usize]=match (mode,part) {
        ("full","field") => &[3000, 3000, 3000, 3000, 3000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 15000, 3600, 3600, 3600, 3600],
        ("full","ring") => &[8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826, 8826],
        ("full","cast") => &[6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000, 6000],
        ("quick","field") => &[3000, 3000, 3000, 3000, 3000, 15000, 15000, 15000, 15000, 15000, 15000, 3600, 3600, 3600, 3600],
        ("quick","ring") => &[7500, 7500, 7500, 7500, 7500, 7500],
        ("quick","cast") => &[5628, 5628, 5628, 5628, 5628, 5628, 5628, 5628],
        _=>return 0,
    };
    let total=durations.iter().sum::<usize>();
    // The warning already expands the ring. Do not replay the small opening sphere
    // inside the open field: quick Void would close before it reached full size.
    let mut time=if part=="field" {
        let opening=durations[..4].iter().sum::<usize>();
        let last=durations.iter().rposition(|d|*d>=12000).unwrap_or(durations.len()-1);
        let hold=durations[4..=last].iter().sum::<usize>();
        opening+elapsed.saturating_sub(expand)*1000%hold
    }
        else {elapsed.min(expand)*total/expand.max(1)};
    for (n,d) in durations.iter().enumerate() {if time<*d {return n;} time-=d;}
    durations.len()-1
}
#[cfg(test)]
mod tests {
    #[test] fn open_domains_use_only_full_size_frames() {
        for (mode,expand,last) in [("full",60,18),("quick",12,10)] {
            assert_eq!(super::frame(mode,"field",expand,expand),4);
            for elapsed in expand..expand+240 {
                assert!((4..=last).contains(&super::frame(mode,"field",elapsed,expand)));
            }
        }
    }
}
