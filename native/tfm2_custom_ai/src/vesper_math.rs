//! Vesper's integer rules, shared 1:1 with the Rift Lab (editor/vesper-math.js) through `vesper_vectors.txt`.
//! Nothing here touches the simulation: chain efficiency, opening delay, Hexcore slots, an athlete's tech modules,
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
    N_FAMILIES * a - a * a.saturating_sub(1) / 2 + (b - a)
}
/// An athlete's Hexcore tuning: the 6 of the 8 tech modules its hexorb slots carry, in slot order.
/// Stable per athlete (a seeded shuffle); `vesper_modules.txt` can override it.
pub fn modules_of(athlete: Option<usize>) -> [usize; 6] {
    let Some(a) = athlete else { return [0, 1, 2, 3, 4, 5] };
    let mut rng = ((a as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x4845_5843_4f52_4500).max(1);
    let mut all = [0usize, 1, 2, 3, 4, 5, 6, 7];
    for i in (1..8).rev() {
        let j = roll(&mut rng, i as u64 + 1) as usize;
        all.swap(i, j);
    }
    [all[0], all[1], all[2], all[3], all[4], all[5]]
}
/// Active hexorb slots at a Hexcore completion (percent).
pub fn hex_slots(completion: i64) -> usize {
    HEX_SLOTS.iter().filter(|(at, _)| completion >= *at).map(|(_, n)| *n).max().unwrap_or(0)
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
        for a in 0..N_FAMILIES {
            for b in a..N_FAMILIES {
                let i = twist_index(a, b);
                assert_eq!(TWISTS[i].a, a);
                assert_eq!(TWISTS[i].b, b);
                assert_eq!(twist_index(b, a), i);
                seen.insert(i);
            }
        }
        assert_eq!(seen.len(), N_TWISTS);
    }
    #[test]
    fn hexcore_slots_and_modules() {
        assert_eq!([69, 70, 74, 75, 80, 85, 89, 90, 100, 130].map(hex_slots), [0, 2, 2, 3, 4, 5, 5, 6, 6, 6]);
        assert_eq!(modules_of(None), [0, 1, 2, 3, 4, 5]);
        for a in 0..200 {
            let m = modules_of(Some(a));
            let mut s = m.to_vec();
            s.sort();
            s.dedup();
            assert_eq!(s.len(), 6);
            assert!(m.iter().all(|x| *x < 8));
            assert_eq!(m, modules_of(Some(a)));
        }
        assert_ne!(modules_of(Some(1)), modules_of(Some(2)));
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
        for athlete in [0usize, 1, 2, 3, 17, 404, 99999] {
            rows.push(format!("modules|{athlete}|{}", modules_of(Some(athlete)).map(|m| m.to_string()).join(",")));
        }
        for c in [0i64, 69, 70, 75, 79, 80, 85, 90, 99, 100, 140] {
            rows.push(format!("slots|{c}|{}", hex_slots(c)));
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
