//! Integer-only resource and stability rules, also implemented by Science Lab.
use std::collections::VecDeque;
pub fn instability(w: &[u32], a: &[u32]) -> Option<u32> {
    if w.is_empty() || w.len() != a.len() || w.contains(&0) || a.contains(&0) {
        return None;
    }
    let wt: u64 = w.iter().map(|x| *x as u64).sum();
    let at: u64 = a.iter().map(|x| *x as u64).sum();
    if at * 4 < wt * 3 {
        return None;
    }
    // Compute the maximum ratio as a fraction; round only once after adding overload.
    let (mut n, mut d) = (0u128, 1u128);
    for (&wi, &ai) in w.iter().zip(a) {
        let nn = (ai as u128 * wt as u128).abs_diff(wi as u128 * at as u128) * 100;
        let dd = wi as u128 * at as u128;
        if nn * d > n * dd {
            n = nn;
            d = dd;
        }
    }
    let over = (at * 4).saturating_sub(wt * 5) as u128 * 20;
    let num = n * wt as u128 + over * d;
    let den = d * wt as u128;
    Some(((num + den / 2) / den).min(100) as u32)
}
pub fn strength(w: &[u32], a: &[u32]) -> u32 {
    let wt: u64 = w.iter().map(|x| *x as u64).sum();
    let at: u64 = a.iter().map(|x| *x as u64).sum();
    if wt == 0 {
        0
    } else {
        (at * 100 / wt).min(125) as u32
    }
}
pub const RECHARGE_TICKS: usize = 300;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Charge {
    Ready,
    Reserved,
    Recovering(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChargeBank {
    pub slots: [Charge; 3],
}
impl Default for ChargeBank {
    fn default() -> Self {
        Self {
            slots: [Charge::Ready; 3],
        }
    }
}
impl ChargeBank {
    pub fn advance(&mut self, tick: usize) {
        for slot in &mut self.slots {
            if matches!(*slot, Charge::Recovering(until) if until <= tick) {
                *slot = Charge::Ready;
            }
        }
    }
    pub fn available(&self) -> usize {
        self.slots.iter().filter(|s| **s == Charge::Ready).count()
    }
    pub fn reserve(&mut self, tick: usize) -> Option<usize> {
        self.advance(tick);
        let slot = self.slots.iter().position(|s| *s == Charge::Ready)?;
        self.slots[slot] = Charge::Reserved;
        Some(slot)
    }
    pub fn commit(&mut self, slot: usize, tick: usize) -> bool {
        if self.slots.get(slot) != Some(&Charge::Reserved) {
            return false;
        }
        self.slots[slot] = Charge::Recovering(tick.saturating_add(RECHARGE_TICKS));
        true
    }
    pub fn cancel(&mut self, slot: usize) {
        if self.slots.get(slot) == Some(&Charge::Reserved) {
            self.slots[slot] = Charge::Ready;
        }
    }
    /// Ten progress steps, ready=10, reserved=11; independent of orb state.
    pub fn cells(&self, tick: usize) -> [usize; 3] {
        self.slots.map(|s| match s {
            Charge::Ready => 10,
            Charge::Reserved => 11,
            Charge::Recovering(until) if until <= tick => 10,
            Charge::Recovering(until) => (RECHARGE_TICKS.saturating_sub(until - tick) / 30).min(9),
        })
    }
}
#[derive(Clone, Default, Debug)]
pub struct Window(pub VecDeque<(usize, usize)>);
impl Window {
    pub fn allow(&mut self, tick: usize, span: usize, cap: usize, n: usize) -> usize {
        while self
            .0
            .front()
            .is_some_and(|(t, _)| tick.saturating_sub(*t) >= span)
        {
            self.0.pop_front();
        }
        let used: usize = self.0.iter().map(|(_, n)| *n).sum();
        let granted = n.min(cap.saturating_sub(used));
        if granted > 0 {
            self.0.push_back((tick, granted));
        }
        granted
    }
}
pub fn unit(dx: i64, dy: i64) -> (i64, i64) {
    let dx = dx.clamp(-2000000, 2000000);
    let dy = dy.clamp(-2000000, 2000000);
    let square = (dx as i128 * dx as i128 + dy as i128 * dy as i128) as u128;
    if square == 0 {
        return (0, 0);
    }
    let square = square * 100000000;
    let mut x = square;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + square / x) / 2;
    }
    (
        (dx as i128 * 100000000 / x as i128) as i64,
        (dy as i128 * 100000000 / x as i128) as i64,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn agreed_vectors() {
        for (a, n) in [
            ([4, 5, 3], 0),
            ([5, 6, 4], 7),
            ([4, 8, 3], 28),
            ([4, 12, 3], 78),
            ([8, 10, 6], 60),
        ] {
            assert_eq!(instability(&[4, 5, 3], &a), Some(n));
        }
        assert_eq!(instability(&[5, 5, 4], &[5, 8, 4]), Some(32));
    }
    #[test]
    fn independent_charge_deadlines_and_exhaustion() {
        let mut p = ChargeBank::default();
        for tick in [0, 30, 60] {
            let slot = p.reserve(tick).unwrap();
            assert!(p.commit(slot, tick));
        }
        assert!(p.reserve(299).is_none());
        assert_eq!(p.cells(150), [5, 4, 3]);
        p.advance(300);
        assert_eq!(p.available(), 1);
        p.advance(329);
        assert_eq!(p.available(), 1);
        p.advance(330);
        assert_eq!(p.available(), 2);
        p.advance(360);
        assert_eq!(p.available(), 3);
    }
    #[test]
    fn cancellation_cannot_refund_spent_or_restart_a_timer() {
        let mut p = ChargeBank::default();
        let slot = p.reserve(20).unwrap();
        p.cancel(slot);
        assert_eq!(p.available(), 3);
        let slot = p.reserve(40).unwrap();
        assert!(p.commit(slot, 40));
        p.cancel(slot);
        assert!(!p.commit(slot, 100));
        p.advance(339);
        assert_eq!(p.available(), 2);
        let cloned = p.clone();
        assert_eq!(cloned, p);
        p.advance(340);
        assert_eq!(p.available(), 3);
    }
    #[test]
    fn rolling_cap() {
        let mut w = Window::default();
        assert_eq!(w.allow(0, 120, 350, 300), 300);
        assert_eq!(w.allow(119, 120, 350, 300), 50);
        assert_eq!(w.allow(120, 120, 350, 300), 300);
    }
    #[test]
    fn invalid_undercharge() {
        assert_eq!(instability(&[4, 5, 3], &[1, 1, 1]), None);
        assert_eq!(instability(&[4, 5, 3], &[4, 0, 5]), None);
    }
    #[test]
    fn vector_normalization() {
        assert_eq!(unit(3000, 4000), (6000, 8000));
        assert_eq!(unit(0, 0), (0, 0));
        assert_eq!(unit(1, 1), (7071, 7071));
        assert_eq!(unit(-3000, 4000), (-6000, 8000));
    }
}
#[cfg(test)]
mod vectors {
    use super::*;
    #[test]
    fn science_vectors() {
        let mut rows = Vec::new();
        for s in crate::unified_theory_data::SKILLS {
            let w = s.charge;
            let cases = vec![
                w.to_vec(),
                w.iter().map(|n| n * 2).collect(),
                w.iter()
                    .enumerate()
                    .map(|(i, n)| if i == 1 { n * 2 } else { *n })
                    .collect(),
                w.iter().map(|_| 1).collect(),
                w.iter().map(|n| n + 1).collect(),
            ];
            for a in cases {
                let ws = w.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
                let aa = a.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
                rows.push(format!(
                    "{ws}|{aa}|{}|{}",
                    instability(w, &a).map_or("invalid".into(), |n| n.to_string()),
                    strength(w, &a)
                ));
            }
        }
        let text = rows.join("\n") + "\n";
        if std::env::var("SCIENCE_VECTORS").as_deref() == Ok("write") {
            std::fs::write(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/unified_theory_vectors.txt"
                ),
                text,
            )
            .unwrap();
        } else {
            assert_eq!(
                text,
                include_str!("unified_theory_vectors.txt").replace("\r\n", "\n")
            );
        }
    }
}
