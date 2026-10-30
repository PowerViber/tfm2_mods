//! Experimental arena bridge. No banish/frozen-target substitute; host interaction acceptance is pending.
use crate::contender_core::{arena, Point, Rect};
use mod_api_stable::{BuffV1, StableSim};
use std::collections::BTreeMap;
const PREFIX: &str = "ct_ring:";
#[cfg(test)]
#[path = "contender_arena_tests.rs"]
mod tests;
#[derive(Clone, Copy, Debug)]
pub struct Ring {
    pub owner: usize,
    pub target: usize,
    pub entries: [Point; 2],
    pub end: usize,
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
        })
    }
    pub fn bounds(self) -> Rect {
        arena(self.owner)
    }
}
pub fn rings(sim: &StableSim) -> Vec<Ring> {
    let mut all = BTreeMap::new();
    for i in 0..sim.champion_count() {
        if let Some(e) = sim.get_entity(sim.champion_id_at(i)) {
            for n in 0..e.buff_count() {
                if let Some(r) = e.buff_at(n).and_then(|b| Ring::parse(b.name())) {
                    all.insert(r.owner, r);
                }
            }
        }
    }
    all.into_values().collect()
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
    };
    let tag = r.tag();
    if tag.len() > mod_api_stable::BUFF_NAME_CAP {
        return false;
    }
    for id in [owner, target] {
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
        }
        return false;
    }
    let bounds = r.bounds();
    let centre = Point {
        x: (bounds.min.x + bounds.max.x) / 2,
        y: (bounds.min.y + bounds.max.y) / 2,
    };
    let starts = [
        Point {
            x: centre.x - crate::contender_core::number("arena", "startingSeparation") / 2,
            y: centre.y,
        },
        Point {
            x: centre.x + crate::contender_core::number("arena", "startingSeparation") / 2,
            y: centre.y,
        },
    ];
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
