//! Optional artwork selection only. No combat state, input, RNG or movement writes.
pub const NIGHT_AURA: &str = "akn_night_aura";
pub const ORB_RENEW: u64 = 3;
pub const ORB_TAIL: u64 = 3;

pub fn orb_phase(tick: u64, born: u64) -> usize {
    (tick.saturating_sub(born) / 15 % 4) as usize
}

pub fn pose(name: &'static str, ticks: u64, dashing: bool) -> &'static str {
    match (name, ticks, dashing) {
        ("skill2", 14, _) => "finale",
        ("skill1", _, true) => "dash",
        ("ult", 14, _) => "night_dash",
        _ => name,
    }
}

pub fn heading(dx: i64, dy: i64) -> &'static str {
    // Presentation-only octant selection; no combat randomness or aim changes.
    let i = ((dy as f64).atan2(dx as f64) / std::f64::consts::FRAC_PI_4).round() as i32;
    ["e", "se", "s", "sw", "w", "nw", "n", "ne"][i.rem_euclid(8) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn renewal_does_not_restart_orb_clock() {
        assert_eq!(orb_phase(100,100),0);assert_eq!(orb_phase(114,100),0);
        assert_eq!(orb_phase(115,100),1);assert_eq!(orb_phase(160,100),0);
        for t in 100..1000 {assert_eq!(orb_phase(t,100),((t-100)/15%4) as usize);}
        assert!(ORB_TAIL<=ORB_RENEW);
    }
    #[test] fn heading_and_pose_select_only_artwork() {
        for ((x,y),n) in [((1,0),"e"),((1,1),"se"),((0,1),"s"),((-1,1),"sw"),((-1,0),"w"),((-1,-1),"nw"),((0,-1),"n"),((1,-1),"ne")] {assert_eq!(heading(x,y),n);}
        assert_eq!(pose("skill1",12,false),"skill1");assert_eq!(pose("skill1",12,true),"dash");
        assert_eq!(pose("skill2",10,false),"skill2");assert_eq!(pose("skill2",14,false),"finale");
        assert_eq!(pose("ult",18,false),"ult");assert_eq!(pose("ult",14,true),"night_dash");
    }
}
