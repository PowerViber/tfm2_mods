//! Vesper's integer rules, shared 1:1 with the Rift Lab (editor/vesper-math.js) through `vesper_vectors.txt`.
//! Nothing here touches the simulation: chain efficiency, opening delay, the seeded atlas and its drift,
//! family-pair bonuses and the private RNG.
use crate::vesper_data::*;

/// The private generator (xorshift64). The Rift Lab ports it with BigInt.
pub fn next(state: &mut u64) -> u64 {
    let mut x = (*state).max(1);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}
pub fn roll(state: &mut u64, n: u64) -> u64 {
    next(state) % n.max(1)
}
/// Squared distance without overflow.
pub fn dist2(ax: i64, ay: i64, bx: i64, by: i64) -> i128 {
    let (dx, dy) = ((ax - bx) as i128, (ay - by) as i128);
    dx * dx + dy * dy
}
/// Integer square root (floor) of a non-negative i128.
pub fn isqrt(v: i128) -> i64 {
    if v <= 0 {
        return 0;
    }
    let mut x = (v as f64).sqrt() as i128;
    while x * x > v {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= v {
        x += 1;
    }
    x as i64
}
pub fn dist(ax: i64, ay: i64, bx: i64, by: i64) -> i64 {
    isqrt(dist2(ax, ay, bx, by))
}
/// Share of a packet's power one link keeps, in basis points: 90% - 30% x (L / scale)^2, clamped to 10%..90%.
pub fn keep_bp(len: i64) -> i64 {
    let l = len.max(0) as i128;
    let s = LINK_SCALE as i128;
    let loss = l * l * (KEEP_QUAD as i128 * 100) / (s * s);
    ((KEEP_BASE * 100) as i128 - loss).clamp((KEEP_MIN * 100) as i128, (KEEP_BASE * 100) as i128) as i64
}
/// Efficiency of a whole chain (the product of every link), in basis points.
pub fn chain_bp(links: &[i64]) -> i64 {
    links.iter().fold(10000, |acc, l| acc * keep_bp(*l) / 10000)
}
/// Ticks a rift needs to open `distance` away from Vesper, for a rift type's delay percentage.
pub fn open_delay(distance: i64, delay_pct: i64) -> usize {
    ((DELAY_BASE + distance.max(0) / DELAY_DIV) * delay_pct / 100).max(1) as usize
}
/// The index of the family-pair bonus for families `a` and `b` (order does not matter).
pub fn twist_index(a: usize, b: usize) -> usize {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    10 * a - a * a.saturating_sub(1) / 2 + (b - a)
}
/// The dimension atlas: each of the 100 dimensions in exactly one 96000-unit cell, plus its drift.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Atlas {
    pub seed: u64,
    pub base: [u8; 100],
}
impl Atlas {
    pub fn new(seed: u64) -> Atlas {
        let mut rng = (seed ^ 0x5645_5350_4552_4154).max(1);
        let mut base = [0u8; 100];
        for (i, b) in base.iter_mut().enumerate() {
            *b = i as u8;
        }
        for i in (1..100).rev() {
            let j = roll(&mut rng, i as u64 + 1) as usize;
            base.swap(i, j);
        }
        Atlas { seed, base }
    }
    /// The whole atlas shift (cells) after `epochs` drifts; each drift moves one cell N, E, S or W.
    pub fn shift(&self, epochs: usize) -> (i64, i64) {
        let mut rng = (self.seed ^ 0x4452_4946_5400).max(1);
        let (mut sx, mut sy) = (0i64, 0i64);
        for _ in 0..epochs {
            match roll(&mut rng, 4) {
                0 => sy -= 1,
                1 => sx += 1,
                2 => sy += 1,
                _ => sx -= 1,
            }
        }
        (sx.rem_euclid(ATLAS_N), sy.rem_euclid(ATLAS_N))
    }
    pub fn epochs(tick: usize) -> usize {
        tick / DRIFT_TICKS as usize
    }
    pub fn cell(x: i64, y: i64) -> (i64, i64) {
        ((x / ATLAS_CELL).clamp(0, ATLAS_N - 1), (y / ATLAS_CELL).clamp(0, ATLAS_N - 1))
    }
    /// The dimension in the cell under (x, y) after `epochs` drifts.
    pub fn dim_at_epoch(&self, x: i64, y: i64, epochs: usize) -> usize {
        let (cx, cy) = Atlas::cell(x, y);
        let (sx, sy) = self.shift(epochs);
        let bx = (cx - sx).rem_euclid(ATLAS_N);
        let by = (cy - sy).rem_euclid(ATLAS_N);
        self.base[(by * ATLAS_N + bx) as usize] as usize
    }
    pub fn dim_at(&self, x: i64, y: i64, tick: usize) -> usize {
        self.dim_at_epoch(x, y, Atlas::epochs(tick))
    }
    /// Where dimension `dim` sits (cell centre) at `tick`.
    pub fn centre_of(&self, dim: usize, tick: usize) -> (i64, i64) {
        let at = self.base.iter().position(|d| *d as usize == dim).unwrap_or(0) as i64;
        let (sx, sy) = self.shift(Atlas::epochs(tick));
        let cx = (at % ATLAS_N + sx).rem_euclid(ATLAS_N);
        let cy = (at / ATLAS_N + sy).rem_euclid(ATLAS_N);
        (cx * ATLAS_CELL + ATLAS_CELL / 2, cy * ATLAS_CELL + ATLAS_CELL / 2)
    }
}
/// Point moved `pct` percent of the way from (ax, ay) toward (bx, by).
pub fn lerp(ax: i64, ay: i64, bx: i64, by: i64, pct: i64) -> (i64, i64) {
    (ax + (bx - ax) * pct / 100, ay + (by - ay) * pct / 100)
}
/// (x, y) moved `len` units along the direction (dx, dy).
pub fn along(x: i64, y: i64, dx: i64, dy: i64, len: i64) -> (i64, i64) {
    let d = isqrt(dx as i128 * dx as i128 + dy as i128 * dy as i128);
    if d == 0 {
        return (x, y);
    }
    (x + dx * len / d, y + dy * len / d)
}
pub fn clamp_map(x: i64, y: i64) -> (i64, i64) {
    (x.clamp(0, MAP), y.clamp(0, MAP))
}
/// Distance from (x, y) to the segment a-b.
pub fn seg_dist(ax: i64, ay: i64, bx: i64, by: i64, x: i64, y: i64) -> i64 {
    let (dx, dy) = ((bx - ax) as i128, (by - ay) as i128);
    let len = dx * dx + dy * dy;
    if len == 0 {
        return dist(ax, ay, x, y);
    }
    let t = (((x - ax) as i128) * dx + ((y - ay) as i128) * dy).clamp(0, len);
    dist(x, y, ax + (dx * t / len) as i64, ay + (dy * t / len) as i64)
}
/// Which side of the line a-b a point is on (-1, 0, 1).
pub fn side(ax: i64, ay: i64, bx: i64, by: i64, x: i64, y: i64) -> i32 {
    let c = (bx - ax) as i128 * (y - ay) as i128 - (by - ay) as i128 * (x - ax) as i128;
    c.signum() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chaining_beats_one_jump() {
        // 180000 units (1.5 link scales): the plan's 900-unit example.
        let one = chain_bp(&[180000]);
        let two = chain_bp(&[90000, 90000]);
        let three = chain_bp(&[60000, 60000, 60000]);
        let four = chain_bp(&[45000, 45000, 45000, 45000]);
        assert_eq!((one, two, three, four), (2250, 5347, 5614, 5415));
        assert!(three > two && three > four && two > one);
        assert_eq!(keep_bp(0), 9000);
        assert_eq!(keep_bp(10_000_000), 1000);
    }
    #[test]
    fn delays_and_twists() {
        assert_eq!(open_delay(0, 100), 6);
        assert_eq!(open_delay(960000, 100), 86);
        assert_eq!(open_delay(960000, 150), 129);
        let mut seen = std::collections::BTreeSet::new();
        for a in 0..10 {
            for b in a..10 {
                let i = twist_index(a, b);
                assert_eq!(TWISTS[i].a, a);
                assert_eq!(TWISTS[i].b, b);
                assert_eq!(twist_index(b, a), i);
                seen.insert(i);
            }
        }
        assert_eq!(seen.len(), 55);
    }
    #[test]
    fn atlas_is_a_permutation_and_drifts() {
        for seed in [1u64, 42, 0xdead_beef, u64::MAX] {
            let a = Atlas::new(seed);
            let mut sorted = a.base.to_vec();
            sorted.sort();
            assert_eq!(sorted, (0..100).collect::<Vec<u8>>());
            for d in [0usize, 37, 99] {
                for tick in [0usize, 1800, 54000] {
                    let (x, y) = a.centre_of(d, tick);
                    assert_eq!(a.dim_at(x, y, tick), d);
                }
            }
        }
        let a = Atlas::new(42);
        assert_eq!(a.shift(0), (0, 0));
        let s1 = a.shift(1);
        assert_eq!((s1.0 + s1.1) % ATLAS_N == 1 || s1 == (ATLAS_N - 1, 0) || s1 == (0, ATLAS_N - 1), true);
        assert_eq!(Atlas::new(42), Atlas::new(42));
        assert_ne!(Atlas::new(42).base, Atlas::new(43).base);
    }
    #[test]
    fn geometry() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(99), 9);
        assert_eq!(dist(0, 0, 3000, 4000), 5000);
        assert_eq!(seg_dist(0, 0, 10000, 0, 5000, 3000), 3000);
        assert_eq!(side(0, 0, 10, 0, 5, 5), 1);
        assert_eq!(side(0, 0, 10, 0, 5, -5), -1);
        assert_eq!(along(0, 0, 3, 4, 5000), (3000, 4000));
    }
}
#[cfg(test)]
mod vectors {
    use super::*;
    /// Parity vectors for the Rift Lab. `VESPER_VECTORS=write` regenerates them.
    #[test]
    fn rift_vectors() {
        let mut rows = Vec::new();
        for links in [
            vec![0],
            vec![60000],
            vec![120000],
            vec![180000],
            vec![90000, 90000],
            vec![60000, 60000, 60000],
            vec![45000, 45000, 45000, 45000],
            vec![36000, 36000, 36000, 36000, 36000],
            vec![250000, 1000, 70000],
        ] {
            let l = links.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
            rows.push(format!("chain|{l}|{}", chain_bp(&links)));
        }
        for d in [0i64, 11999, 12000, 480000, 960000, 1357645] {
            for pct in [100, 150] {
                rows.push(format!("delay|{d}|{pct}|{}", open_delay(d, pct)));
            }
        }
        for seed in [1u64, 42, 7777, 0x1234_5678_9abc_def0] {
            let a = Atlas::new(seed);
            rows.push(format!(
                "atlas|{seed}|{}",
                a.base.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(",")
            ));
            for e in [0usize, 1, 2, 5, 17, 60] {
                let (sx, sy) = a.shift(e);
                rows.push(format!("drift|{seed}|{e}|{sx},{sy}"));
            }
        }
        let mut rng = 99u64;
        let draws: Vec<String> = (0..12).map(|_| roll(&mut rng, 1000).to_string()).collect();
        rows.push(format!("rng|99|{}", draws.join(",")));
        let text = rows.join("\n") + "\n";
        if std::env::var("VESPER_VECTORS").as_deref() == Ok("write") {
            std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/src/vesper_vectors.txt"), text).unwrap();
        } else {
            assert_eq!(text, include_str!("vesper_vectors.txt").replace("\r\n", "\n"));
        }
    }
}
