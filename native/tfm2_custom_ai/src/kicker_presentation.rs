//! Read-only presentation planning: no damage, movement, input, CC or RNG writes.
use crate::kicker_core::{Core, Frame, MoveKind, Point};
use crate::kicker_art_data::{HEADINGS, MAX_RENEW_TICKS};

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub name: String,
    pub at: Point,
}
fn heading(a: Point, b: Point) -> &'static str {
    let angle = ((b.y - a.y) as f64).atan2((b.x - a.x) as f64);
    let n = (angle / std::f64::consts::FRAC_PI_4).round() as i32;
    HEADINGS[n.rem_euclid(8) as usize]
}
/// Uses the already-built frame and accepted core state, never another world scan.
pub fn cues(core: &Core, f: &Frame) -> Vec<Cue> {
    if !f.alive || (core.rocks.is_empty() && core.flights.is_empty() && core.ult.is_none()
        && !core.moving.as_ref().is_some_and(|m| m.kind == MoveKind::Skyfall)) { return vec![]; }
    let mut out = Vec::with_capacity(8);
    for rock in &core.rocks {
        out.push(Cue { name: format!("kk_rock_{}", heading(rock.from, rock.end)), at: rock.pos });
    }
    for flight in &core.flights {
        if let Some(u) = f.unit(flight.target).filter(|u| u.visible && u.hp > 0) {
            out.push(Cue { name: format!("kk_kick_trail_{}", heading(flight.from, flight.to)), at: u.pos });
        }
    }
    if let Some(m) = core.moving.as_ref().filter(|m| m.kind == MoveKind::Skyfall) {
        out.push(Cue { name: format!("kk_skyfall_{}", heading(f.pos, m.land)), at: f.pos });
    }
    if let Some(w) = &core.ult {
        out.push(Cue { name: "kk_tag".into(), at: w.anchor });
    }
    out
}
/// One short single-frame lease per update, round-robin. Shares the bridge's
/// existing token bucket; critical impacts retain its four-token ambient reserve.
#[derive(Clone, Debug, Default)]
pub struct Presenter { cursor: usize }
impl Presenter {
    pub fn next(&mut self, core: &Core, f: &Frame) -> Option<(Cue, u64)> {
        let list = cues(core, f);
        if list.is_empty() { self.cursor = 0; return None; }
        let ttl = (list.len() as u64 + 2).clamp(3, MAX_RENEW_TICKS);
        let cue = list[self.cursor % list.len()].clone();
        self.cursor = self.cursor.wrapping_add(1);
        Some((cue, ttl))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::kicker_core::{Rock, SecondWind};
    fn frame() -> Frame {
        serde_json::from_value(serde_json::json!({"tick":50,"owner":1,"team":0,"alive":true,
            "pos":{"x":300000,"y":300000},"hp":1500,"max_hp":1500,"attack":150,
            "stunned":false,"ult_unlocked":true,"units":[],"profile":[50,50,50,50,50,50,50,50,50,50,50,50,50],"manual":true})).unwrap()
    }
    #[test]
    fn travelling_art_uses_current_core_position_and_fixed_direction() {
        let f=frame();let mut c=Core::default();
        c.rocks.push(Rock{from:f.pos,end:Point::new(400000,300000),pos:Point::new(320000,300000),attack:150,pct:100,hit:vec![]});
        let before=serde_json::to_string(&c).unwrap();
        let cue=cues(&c,&f);assert_eq!(cue[0].name,"kk_rock_e");assert_eq!(cue[0].at,c.rocks[0].pos);
        assert_eq!(serde_json::to_string(&c).unwrap(),before);
    }
    #[test]
    fn anchor_lifetime_death_and_owners_do_not_leak() {
        let mut f=frame();let mut a=Core::default();let b=Core::default();
        a.ult=Some(SecondWind{anchor:Point::new(200000,200000),hp:1500,until:500,roof:false});
        let mut pa=Presenter::default();let mut pb=Presenter::default();
        assert_eq!(pa.next(&a,&f).unwrap().0.at,a.ult.as_ref().unwrap().anchor);
        assert!(pb.next(&b,&f).is_none());
        a.ult=None;assert!(pa.next(&a,&f).is_none());
        f.alive=false;assert!(cues(&a,&f).is_empty());
    }
    #[test]
    fn all_eight_headings_and_short_leases_are_bounded() {
        let a=Point::new(300000,300000);
        for (i,(x,y)) in [(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1),(0,-1),(1,-1)].into_iter().enumerate() {
            assert_eq!(heading(a,Point::new(a.x+x*1000,a.y+y*1000)),HEADINGS[i]);
        }
        let f=frame();let mut c=Core::default();c.ult=Some(SecondWind{anchor:a,hp:1500,until:500,roof:false});
        assert!(Presenter::default().next(&c,&f).unwrap().1<=MAX_RENEW_TICKS);
    }
}
