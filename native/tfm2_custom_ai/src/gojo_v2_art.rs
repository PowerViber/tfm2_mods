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
    let mut time=if part=="field" {elapsed.saturating_sub(expand)*1000%total}
        else {elapsed.min(expand)*total/expand.max(1)};
    for (n,d) in durations.iter().enumerate() {if time<*d {return n;} time-=d;}
    durations.len()-1
}
