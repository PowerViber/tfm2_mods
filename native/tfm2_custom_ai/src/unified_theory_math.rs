//! Integer helpers shared by the native champions: a rolling-window limiter and unit vectors.
use std::collections::VecDeque;
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
    fn rolling_cap() {
        let mut w = Window::default();
        assert_eq!(w.allow(0, 120, 350, 300), 300);
        assert_eq!(w.allow(119, 120, 350, 300), 50);
        assert_eq!(w.allow(120, 120, 350, 300), 300);
    }
    #[test]
    fn vector_normalization() {
        assert_eq!(unit(3000, 4000), (6000, 8000));
        assert_eq!(unit(0, 0), (0, 0));
        assert_eq!(unit(1, 1), (7071, 7071));
        assert_eq!(unit(-3000, 4000), (-6000, 8000));
    }
}
