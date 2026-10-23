//! Conservative public-terrain routing. One 30x30 flood per target, shared by every athlete.
use crate::strategic_core::{Actor, Point, Target};
use std::collections::{BTreeMap, VecDeque};
pub type Routes = BTreeMap<(usize, usize), Option<i64>>;
const N: usize = 30;
const CELL: i64 = 32000;
fn centre(n: usize) -> Point {
    (
        (n % N) as i64 * CELL + CELL / 2,
        (n / N) as i64 * CELL + CELL / 2,
    )
}
fn cell(p: Point) -> usize {
    ((p.1 / CELL).clamp(0, 29) as usize) * N + (p.0 / CELL).clamp(0, 29) as usize
}
pub fn build(actors: &[Actor], targets: &[Target], wall: &dyn Fn(Point) -> bool) -> Routes {
    let blocked: Vec<_> = (0..N * N).map(|n| wall(centre(n))).collect();
    let nearest = |p: Point| -> Option<usize> {
        let id = cell(p);
        if !blocked[id] {
            return Some(id);
        }
        let (x, y) = (id % N, id / N);
        (y.saturating_sub(1)..=(y + 1).min(N - 1))
            .flat_map(|yy| (x.saturating_sub(1)..=(x + 1).min(N - 1)).map(move |xx| yy * N + xx))
            .filter(|n| !blocked[*n])
            .min_by_key(|n| crate::strategic_core::distance(p, centre(*n)))
    };
    let mut out = Routes::new();
    for t in targets.iter().take(32) {
        let mut d = vec![i64::MAX; N * N];
        let mut queue = VecDeque::new();
        if let Some(c) = nearest(t.point) {
            d[c] = crate::strategic_core::distance(t.point, centre(c));
            queue.push_back(c);
        }
        while let Some(c) = queue.pop_front() {
            let (x, y) = (c % N, c / N);
            for n in [
                x.checked_sub(1).map(|x| y * N + x),
                (x + 1 < N).then_some(y * N + x + 1),
                y.checked_sub(1).map(|y| y * N + x),
                (y + 1 < N).then_some((y + 1) * N + x),
            ]
            .into_iter()
            .flatten()
            {
                if !blocked[n] && d[n] == i64::MAX {
                    d[n] = d[c] + CELL;
                    queue.push_back(n);
                }
            }
        }
        for a in actors {
            let direct = crate::strategic_core::distance(a.point, t.point);
            let clear = (0..=32).all(|n| {
                !wall((
                    a.point.0 + (t.point.0 - a.point.0) * n / 32,
                    a.point.1 + (t.point.1 - a.point.1) * n / 32,
                ))
            });
            let route = if clear {
                Some(direct)
            } else {
                nearest(a.point)
                    .filter(|c| d[*c] != i64::MAX)
                    .map(|c| d[c] + crate::strategic_core::distance(a.point, centre(c)))
            };
            out.insert((a.id, t.id), route);
        }
    }
    out
}
/// The next cell centre to walk to from `from` toward `to` around walls (a breadth-first search over the same
/// 30x30 grid): None when `from`'s cell already is the target's, or when no route exists.
pub fn next_step(from: Point, to: Point, wall: &dyn Fn(Point) -> bool) -> Option<Point> {
    let blocked: Vec<_> = (0..N * N).map(|n| wall(centre(n))).collect();
    let open = |p: Point| -> Option<usize> {
        let id = cell(p);
        if !blocked[id] {
            return Some(id);
        }
        let (x, y) = (id % N, id / N);
        (y.saturating_sub(1)..=(y + 1).min(N - 1))
            .flat_map(|yy| (x.saturating_sub(1)..=(x + 1).min(N - 1)).map(move |xx| yy * N + xx))
            .filter(|n| !blocked[*n])
            .min_by_key(|n| (crate::strategic_core::distance(p, centre(*n)), *n))
    };
    let (start, goal) = (open(from)?, open(to)?);
    if start == goal {
        return None;
    }
    let neighbours = |c: usize| {
        let (x, y) = (c % N, c / N);
        [
            x.checked_sub(1).map(|x| y * N + x),
            (x + 1 < N).then_some(y * N + x + 1),
            y.checked_sub(1).map(|y| y * N + x),
            (y + 1 < N).then_some((y + 1) * N + x),
        ]
        .into_iter()
        .flatten()
    };
    let mut d = vec![usize::MAX; N * N];
    let mut queue = VecDeque::from([goal]);
    d[goal] = 0;
    while let Some(c) = queue.pop_front() {
        for n in neighbours(c) {
            if !blocked[n] && d[n] == usize::MAX {
                d[n] = d[c] + 1;
                queue.push_back(n);
            }
        }
    }
    if d[start] == usize::MAX {
        return None;
    }
    neighbours(start)
        .filter(|n| d[*n] < d[start])
        .min_by_key(|n| (d[*n], *n))
        .map(centre)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocked_route_never_invents_an_arrival() {
        let wall = |p: Point| p.0 >= 320000 && p.0 < 352000;
        let actors = [Actor {
            id: 1,
            athlete: 1,
            team: 0,
            point: (200000, 400000),
            hp: 1,
            max_hp: 1,
            speed: 1000,
            power: 1,
            range: 1,
            role: 0,
            controlled: false,
            ultimate: false,
            visible: true,
            ratings: Default::default(),
            judgement: 50,
            ego: 50,
        }];
        let targets = [Target {
            id: 2,
            kind: 2,
            team: 2,
            point: (500000, 400000),
            hp: 1,
            max_hp: 1,
            loss: 0,
            visible: true,
            value: 1,
        }];
        assert_eq!(build(&actors, &targets, &wall)[&(1, 2)], None);
        let opening = |p: Point| wall(p) && p.1 < 800000;
        assert!(build(&actors, &targets, &opening)[&(1, 2)].unwrap() > 700000);
    }
}
