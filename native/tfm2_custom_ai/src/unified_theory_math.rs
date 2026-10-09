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
#[derive(Clone, Debug)]
pub struct Pool {
    pub free: u32,
    pub reserved: u32,
    remainder: u32,
}
impl Default for Pool {
    fn default() -> Self {
        Self {
            free: 10000,
            reserved: 0,
            remainder: 0,
        }
    }
}
impl Pool {
    pub fn reserve(&mut self, n: u32) -> bool {
        if n > self.free {
            return false;
        }
        self.free -= n;
        self.reserved += n;
        true
    }
    pub fn commit(&mut self, n: u32) -> bool {
        if n > self.reserved {
            return false;
        }
        self.reserved -= n;
        true
    }
    pub fn refund(&mut self, n: u32) {
        let n = n.min(self.reserved);
        self.reserved -= n;
        self.free += n;
    }
    pub fn regen(&mut self, combat: bool) {
        self.remainder += if combat { 400 } else { 800 };
        let n = self.remainder / 60;
        self.remainder %= 60;
        self.free = (self.free + n).min(10000 - self.reserved);
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
    fn reservation_conservation() {
        let mut p = Pool::default();
        assert!(p.reserve(9000));
        for _ in 0..600 {
            p.regen(false);
        }
        assert_eq!(p.free + p.reserved, 10000);
        assert!(p.commit(1000));
        p.refund(8000);
        assert_eq!(p.free, 9000);
        assert!(!p.commit(1));
    }
    #[test]
    fn integer_regeneration() {
        let mut p = Pool {
            free: 0,
            ..Pool::default()
        };
        for _ in 0..60 {
            p.regen(true);
        }
        assert_eq!(p.free, 400);
        for _ in 0..60 {
            p.regen(false);
        }
        assert_eq!(p.free, 1200);
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
            assert_eq!(text, include_str!("unified_theory_vectors.txt"));
        }
    }
}
