//! All geometry uses integers, including the sampled arcs; native/WASM use identical paths.
use crate::steve_craft_data::{MAP, SIN, WALL_HALF, WALL_LENGTH};
pub type Point = (i64, i64);
pub fn square(p: Point, q: Point) -> i64 {
    let d = (p.0 - q.0, p.1 - q.1);
    d.0 * d.0 + d.1 * d.1
}
pub fn length(p: Point, q: Point) -> i64 {
    sqrt(square(p, q).max(0) as u64) as i64
}
pub fn sqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let (mut a, mut b) = (n, n / 2 + 1);
    while b < a {
        a = b;
        b = (a + n / a) / 2;
    }
    a
}
pub fn within(p: Point, q: Point, r: i64) -> bool {
    square(p, q) <= r * r
}
pub fn legal(p: Point) -> bool {
    p.0 >= 0 && p.1 >= 0 && p.0 < MAP && p.1 < MAP
}
pub fn toward(a: Point, b: Point, d: i64) -> Point {
    let l = length(a, b).max(1);
    (
        a.0 + (b.0 - a.0) * d.min(l) / l,
        a.1 + (b.1 - a.1) * d.min(l) / l,
    )
}
pub fn distance_segment(p: Point, a: Point, b: Point) -> i64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let n = (dx * dx + dy * dy).max(1);
    let t = ((p.0 - a.0) * dx + (p.1 - a.1) * dy).clamp(0, n);
    length(p, (a.0 + dx * t / n, a.1 + dy * t / n))
}
pub fn crosses(a: Point, b: Point, c: Point, d: Point) -> bool {
    fn orient(a: Point, b: Point, c: Point) -> i128 {
        (b.0 - a.0) as i128 * (c.1 - a.1) as i128 - (b.1 - a.1) as i128 * (c.0 - a.0) as i128
    }
    let (o1, o2, o3, o4) = (
        orient(a, b, c),
        orient(a, b, d),
        orient(c, d, a),
        orient(c, d, b),
    );
    if o1 == 0 && o2 == 0 {
        return a.0.min(b.0) <= c.0.max(d.0)
            && c.0.min(d.0) <= a.0.max(b.0)
            && a.1.min(b.1) <= c.1.max(d.1)
            && c.1.min(d.1) <= a.1.max(b.1);
    }
    (o1 == 0 || o2 == 0 || o1.signum() != o2.signum())
        && (o3 == 0 || o4 == 0 || o3.signum() != o4.signum())
}
pub fn clear(a: Point, b: Point, wall: &dyn Fn(i64, i64) -> bool) -> bool {
    let n = (length(a, b) / 2000).max(1);
    (0..=n).all(|i| !wall(a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n))
}
pub fn clip(a: Point, b: Point, wall: &dyn Fn(i64, i64) -> bool) -> Point {
    let n = (length(a, b) / 2000).max(1);
    let mut last = a;
    for i in 1..=n {
        let p = (a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n);
        if !legal(p) || wall(p.0, p.1) {
            break;
        }
        last = p;
    }
    last
}
/// `bend` -4..4 selects signed 45-degree increments; heading is one of sixteen directions.
pub fn arc(centre: Point, heading: u8, bend: i8) -> Vec<Point> {
    let h = (heading as usize % 16) * 16;
    let (ux, uy) = (SIN[(h + 64) % 256], SIN[h]);
    if bend == 0 {
        return vec![
            (
                centre.0 - ux * WALL_LENGTH / (2 * (length((0, 0), (ux, uy)) + 1)),
                centre.1 - uy * WALL_LENGTH / (2 * (length((0, 0), (ux, uy)) + 1)),
            ),
            (
                centre.0 + ux * WALL_LENGTH / (2 * (length((0, 0), (ux, uy)) + 1)),
                centre.1 + uy * WALL_LENGTH / (2 * (length((0, 0), (ux, uy)) + 1)),
            ),
        ];
    }
    let span = (bend.unsigned_abs().min(4) as i64) * 32;
    let half = span / 2;
    let cy = SIN[((half + 64) % 256) as usize];
    let sign = bend.signum() as i64;
    let raw: Vec<Point> = (0..=16)
        .map(|i| {
            let a = (-half + span * i / 16).rem_euclid(256) as usize;
            (SIN[a], (SIN[(a + 64) % 256] - cy) * sign)
        })
        .collect();
    let total = raw
        .windows(2)
        .map(|w| length(w[0], w[1]))
        .sum::<i64>()
        .max(1);
    let mut out: Vec<_> = raw
        .into_iter()
        .map(|(x, y)| {
            let x = x * WALL_LENGTH / total;
            let y = y * WALL_LENGTH / total;
            (
                centre.0 + (ux * x - uy * y) / 10000,
                centre.1 + (uy * x + ux * y) / 10000,
            )
        })
        .collect();
    // Per-chord integer roots round down. Clamp the *final* sampled path as well.
    let measured = path_length(&out);
    if measured > WALL_LENGTH {
        for p in &mut out {
            p.0 = centre.0 + (p.0 - centre.0) * (WALL_LENGTH - 32) / measured;
            p.1 = centre.1 + (p.1 - centre.1) * (WALL_LENGTH - 32) / measured;
        }
    }
    out
}
pub fn path_length(p: &[Point]) -> i64 {
    p.windows(2).map(|w| length(w[0], w[1])).sum()
}
pub fn at(p: &[Point], mut distance: i64) -> (Point, usize) {
    for (i, w) in p.windows(2).enumerate() {
        let l = length(w[0], w[1]);
        if distance <= l {
            return (toward(w[0], w[1], distance), i);
        }
        distance -= l;
    }
    (*p.last().unwrap_or(&(0, 0)), p.len().saturating_sub(2))
}
pub fn blocked(a: Point, b: Point, segments: &[(Point, Point)]) -> bool {
    segments.iter().any(|&(c, d)| {
        crosses(a, b, c, d)
            || [
                distance_segment(a, c, d),
                distance_segment(b, c, d),
                distance_segment(c, a, b),
                distance_segment(d, a, b),
            ]
            .into_iter()
            .min()
            .unwrap()
                < WALL_HALF + 2000
    })
}
/// Visibility graph routes around the union of connected segments, with deterministic Dijkstra ties.
pub fn detour(
    a: Point,
    b: Point,
    segs: &[(Point, Point)],
    wall: &dyn Fn(i64, i64) -> bool,
) -> Option<Point> {
    if !blocked(a, b, segs) && clear(a, b, wall) {
        return None;
    }
    let mut nodes = vec![a, b];
    for &(c, d) in segs.iter().take(96) {
        for p in [c, d] {
            for (dx, dy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let q = (p.0 + dx * (WALL_HALF + 5000), p.1 + dy * (WALL_HALF + 5000));
                if legal(q)
                    && !wall(q.0, q.1)
                    && !segs
                        .iter()
                        .any(|&(v, w)| distance_segment(q, v, w) < WALL_HALF + 1000)
                    && !nodes.contains(&q)
                {
                    nodes.push(q)
                }
            }
        }
    }
    let n = nodes.len();
    let (mut costs, mut prev, mut done) = (vec![i64::MAX; n], vec![None; n], vec![false; n]);
    costs[0] = 0;
    for _ in 0..n {
        let Some(i) = (0..n)
            .filter(|&i| !done[i] && costs[i] != i64::MAX)
            .min_by_key(|&i| (costs[i], i))
        else {
            break;
        };
        if i == 1 {
            break;
        }
        done[i] = true;
        for j in 1..n {
            if done[j] || blocked(nodes[i], nodes[j], segs) || !clear(nodes[i], nodes[j], wall) {
                continue;
            }
            let cost = costs[i] + length(nodes[i], nodes[j]);
            if cost < costs[j] {
                costs[j] = cost;
                prev[j] = Some(i);
            }
        }
    }
    if prev[1].is_none() {
        return Some(a);
    }
    let mut j = 1;
    while let Some(i) = prev[j] {
        if i == 0 {
            return Some(nodes[j]);
        }
        j = i;
    }
    Some(a)
}
