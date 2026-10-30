//! Main Event ring bridge. No banish/frozen-target substitute.
//!
//! 0.10.81: the ring is ON the map. The first live test put both fighters at x 1,120,000, outside the 960,000 map;
//! the engine accepted the move and then pulled them back to the east edge, where they stayed stuck for the whole
//! duel. The ring is now a square centred on the duel (clear of walls where possible), with the exact entry-point
//! return unchanged. Other champions are moved out of the ring while it stands.
use crate::contender_core::{map, number, Point, Rect};
use mod_api_stable::{BuffV1, StableSim};
use std::collections::BTreeMap;
const PREFIX: &str = "ct_ring:";
const CENTRE: &str = "ct_ctr:";
#[cfg(test)]
#[path = "contender_arena_tests.rs"]
mod tests;
#[derive(Clone, Copy, Debug)]
pub struct Ring {
    pub owner: usize,
    pub target: usize,
    pub entries: [Point; 2],
    pub end: usize,
    /// The ring's centre on the map (stored beside the ledger in a `ct_ctr:` buff).
    pub centre: Point,
}
impl Ring {
    fn tag(self) -> String {
        format!(
            "{PREFIX}{}:{}:{}:{}:{}:{}:{}",
            self.owner,
            self.target,
            self.entries[0].x,
            self.entries[0].y,
            self.entries[1].x,
            self.entries[1].y,
            self.end
        )
    }
    fn parse(tag: &str) -> Option<Self> {
        let v: Vec<_> = tag
            .strip_prefix(PREFIX)?
            .split(':')
            .map(str::parse::<u64>)
            .collect::<Result<_, _>>()
            .ok()?;
        if v.len() != 7 || v[0] == v[1] {
            return None;
        }
        Some(Self {
            owner: v[0] as usize,
            target: v[1] as usize,
            entries: [
                Point {
                    x: v[2] as i64,
                    y: v[3] as i64,
                },
                Point {
                    x: v[4] as i64,
                    y: v[5] as i64,
                },
            ],
            end: v[6] as usize,
            centre: Point::default(),
        })
        .map(|mut r| {
            r.centre = midpoint(r.entries);
            r
        })
    }
    fn centre_tag(self) -> String {
        format!("{CENTRE}{}:{}:{}", self.owner, self.centre.x, self.centre.y)
    }
    pub fn bounds(self) -> Rect {
        square(self.centre)
    }
}
fn half() -> i64 {
    number("arena", "interiorSide") / 2
}
fn square(c: Point) -> Rect {
    Rect {
        min: Point { x: c.x - half(), y: c.y - half() },
        max: Point { x: c.x + half(), y: c.y + half() },
    }
}
fn midpoint(e: [Point; 2]) -> Point {
    Point { x: (e[0].x + e[1].x) / 2, y: (e[0].y + e[1].y) / 2 }
}
fn wall_cells(r: Rect) -> usize {
    let cell = crate::walls::CELL;
    let mut n = 0;
    let mut y = r.min.y + cell / 2;
    while y < r.max.y {
        let mut x = r.min.x + cell / 2;
        while x < r.max.x {
            n += crate::walls::wall_at(x, y) as usize;
            x += cell;
        }
        y += cell;
    }
    n
}
/// The ring's centre: the duel's midpoint, kept inside the map, moved up to three cells to the spot with the
/// fewest walls inside the square (ties keep the nearest).
pub fn place(entries: [Point; 2]) -> Point {
    let m = map();
    let inside = |c: Point| Point {
        x: c.x.clamp(m.min.x + half(), m.max.x - half()),
        y: c.y.clamp(m.min.y + half(), m.max.y - half()),
    };
    let mid = inside(midpoint(entries));
    let cell = crate::walls::CELL;
    let mut best = (wall_cells(square(mid)), 0i64, mid);
    for dy in -3..=3i64 {
        for dx in -3..=3i64 {
            let c = inside(Point { x: mid.x + dx * cell, y: mid.y + dy * cell });
            let key = (wall_cells(square(c)), dx * dx + dy * dy, c);
            if (key.0, key.1) < (best.0, best.1) {
                best = key;
            }
        }
    }
    best.2
}
/// The nearest point to `p` that is not inside a wall cell (searching outward a cell at a time).
fn free(p: Point, within: Rect, r: i64) -> Point {
    let p = within.clamp(p, r);
    if !crate::walls::wall_at(p.x, p.y) {
        return p;
    }
    let cell = crate::walls::CELL;
    for ring in 1..=4i64 {
        for dy in -ring..=ring {
            for dx in -ring..=ring {
                if dx.abs().max(dy.abs()) != ring {
                    continue;
                }
                let q = within.clamp(Point { x: p.x + dx * cell, y: p.y + dy * cell }, r);
                if !crate::walls::wall_at(q.x, q.y) {
                    return q;
                }
            }
        }
    }
    p
}
pub fn rings(sim: &StableSim) -> Vec<Ring> {
    let mut all = BTreeMap::new();
    let mut centres = BTreeMap::new();
    for i in 0..sim.champion_count() {
        if let Some(e) = sim.get_entity(sim.champion_id_at(i)) {
            for n in 0..e.buff_count() {
                let Some(b) = e.buff_at(n) else { continue };
                if let Some(r) = Ring::parse(b.name()) {
                    all.insert(r.owner, r);
                } else if let Some(v) = b.name().strip_prefix(CENTRE) {
                    let v: Vec<i64> = v.split(':').filter_map(|x| x.parse().ok()).collect();
                    if let [owner, x, y] = v[..] {
                        centres.insert(owner as usize, Point { x, y });
                    }
                }
            }
        }
    }
    all.into_values()
        .map(|mut r| {
            r.centre = centres.get(&r.owner).copied().unwrap_or_else(|| place(r.entries));
            r
        })
        .collect()
}
pub fn member(sim: &StableSim, id: usize) -> Option<Ring> {
    rings(sim)
        .into_iter()
        .find(|r| r.owner == id || r.target == id)
}
pub fn same(sim: &StableSim, a: usize, b: usize) -> bool {
    member(sim, a).map(|r| r.owner) == member(sim, b).map(|r| r.owner)
}
pub fn available(sim: &StableSim, id: usize) -> bool {
    member(sim, id).is_none()
        && sim.get_entity(id).is_some_and(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .all(|b| !b.name().starts_with("ct_reserve:"))
        })
}
pub fn reserved_rect(sim: &StableSim, rect: Rect) -> bool {
    rings(sim).iter().flat_map(|r| r.entries).any(|p| {
        let x = p.x.clamp(rect.min.x, rect.max.x);
        let y = p.y.clamp(rect.min.y, rect.max.y);
        p.distance2(Point { x, y }) <= 10000_i128.pow(2)
    })
}
pub fn reserve(sim: &mut StableSim, owner: usize, target: usize, ticks: usize) -> bool {
    if !available(sim, owner) || !available(sim, target) {
        return false;
    }
    let tag = format!("ct_reserve:{owner}");
    for id in [owner, target] {
        sim.add_buff(id, &crate::timed(&tag, ticks));
    }
    true
}
pub fn release_reservation(sim: &mut StableSim, owner: usize, target: usize) {
    for id in [owner, target] {
        sim.entity_remove_buff(id, &format!("ct_reserve:{owner}"));
    }
}
fn alive(sim: &StableSim, id: usize) -> bool {
    sim.get_entity(id).is_some_and(|e| e.is_alive())
}
fn set(sim: &mut StableSim, id: usize, p: Point) -> bool {
    sim.entity_set_pos(id, p.x.max(0) as u64, p.y.max(0) as u64)
        && sim
            .get_entity(id)
            .is_some_and(|e| e.pos() == (p.x as u64, p.y as u64))
}
pub fn enter(sim: &mut StableSim, owner: usize, target: usize, duration: usize) -> bool {
    if !alive(sim, owner)
        || !alive(sim, target)
        || member(sim, owner).is_some()
        || member(sim, target).is_some()
    {
        return false;
    }
    let entries = [owner, target].map(|id| {
        let p = sim.get_entity(id).unwrap().pos();
        Point {
            x: p.0 as i64,
            y: p.1 as i64,
        }
    });
    let r = Ring {
        owner,
        target,
        entries,
        end: sim.tick() + duration,
        centre: place(entries),
    };
    let tag = r.tag();
    if tag.len() > mod_api_stable::BUFF_NAME_CAP || r.centre_tag().len() > mod_api_stable::BUFF_NAME_CAP {
        return false;
    }
    for id in [owner, target] {
        sim.add_buff(id, &BuffV1::named(&r.centre_tag()));
        sim.add_buff(id, &BuffV1::named(&tag));
    }
    if ![owner, target].iter().all(|&id| {
        sim.get_entity(id).is_some_and(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .any(|b| b.name() == tag)
        })
    }) {
        for id in [owner, target] {
            sim.entity_remove_buff(id, &tag);
            sim.entity_remove_buff(id, &r.centre_tag());
        }
        return false;
    }
    let bounds = r.bounds();
    let centre = Point {
        x: (bounds.min.x + bounds.max.x) / 2,
        y: (bounds.min.y + bounds.max.y) / 2,
    };
    let r_body = number("arena", "defaultChampionRadius");
    let starts = [
        Point {
            x: centre.x - number("arena", "startingSeparation") / 2,
            y: centre.y,
        },
        Point {
            x: centre.x + number("arena", "startingSeparation") / 2,
            y: centre.y,
        },
    ]
    .map(|p| free(p, bounds, r_body));
    let accepted = set(sim, owner, starts[0]) && set(sim, target, starts[1]);
    release_reservation(sim, owner, target);
    if !accepted {
        if !restore(sim, r) {
            let retry = Ring {
                end: sim.tick(),
                ..r
            };
            for id in [owner, target] {
                sim.entity_remove_buff(id, &tag);
                sim.add_buff(id, &BuffV1::named(&retry.tag()));
            }
            // the retry ledger keeps its centre buff
        }
        return false;
    }
    true
}
fn restore(sim: &mut StableSim, r: Ring) -> bool {
    let mut success = true;
    for (id, p) in [r.owner, r.target].into_iter().zip(r.entries) {
        if alive(sim, id)
            && sim
                .get_entity(id)
                .is_some_and(|e| e.pos() != (p.x as u64, p.y as u64))
        {
            success &= set(sim, id, p)
        }
    }
    if success {
        for id in [r.owner, r.target] {
            sim.entity_remove_buff(id, &r.tag());
            sim.entity_remove_buff(id, &r.centre_tag());
        }
    }
    success
}
/// Runs outside owner passives so either death still returns the survivor.
pub fn tick(sim: &mut StableSim) {
    for r in rings(sim) {
        if sim.tick() >= r.end || !alive(sim, r.owner) || !alive(sim, r.target) || sim.is_end() {
            restore(sim, r);
            continue;
        }
        if sim.tick() % 5 == 0 {
            let b = r.bounds();
            crate::fx_point(
                sim,
                "ct_ring",
                r.owner,
                (b.min.x + b.max.x) / 2,
                (b.min.y + b.max.y) / 2,
                5,
            );
            for (p, name) in r
                .entries
                .into_iter()
                .zip(["ct_return_red", "ct_return_blue"])
            {
                crate::fx_point(sim, name, r.owner, p.x, p.y, 5);
            }
        }
        // Others may not walk into a standing ring: move them to the nearest point just outside its edge.
        let b = r.bounds();
        let outsiders: Vec<(usize, Point, i64)> = (0..sim.champion_count())
            .map(|i| sim.champion_id_at(i))
            .filter(|&id| id != r.owner && id != r.target)
            .filter_map(|id| sim.get_entity(id).filter(|e| e.is_alive()).map(|e| {
                let q = e.pos();
                (id, Point { x: q.0 as i64, y: q.1 as i64 }, e.radius().max(1) as i64)
            }))
            .filter(|(id, p, rad)| member(sim, *id).is_none() && b.contains(*p, -rad))
            .collect();
        for (id, p, rad) in outsiders {
            let m = rad + 2000;
            let exits = [
                Point { x: b.min.x - m, y: p.y },
                Point { x: b.max.x + m, y: p.y },
                Point { x: p.x, y: b.min.y - m },
                Point { x: p.x, y: b.max.y + m },
            ];
            let out = exits
                .into_iter()
                .filter(|q| map().contains(*q, rad) && !crate::walls::wall_at(q.x, q.y))
                .min_by_key(|q| q.distance2(p));
            if let Some(q) = out {
                set(sim, id, q);
            }
        }
        for id in [r.owner, r.target] {
            if let Some(e) = sim.get_entity(id) {
                let q = e.pos();
                let p = Point {
                    x: q.0 as i64,
                    y: q.1 as i64,
                };
                let radius = e.radius().max(1) as i64;
                let b = r.bounds();
                if !b.contains(p, radius) {
                    set(sim, id, b.clamp(p, radius));
                }
            }
        }
    }
}
