//! Main Event ring bridge. No banish/frozen-target substitute.
//!
//! 0.10.81: the ring is ON the map. The first live test put both fighters at x 1,120,000, outside the 960,000 map;
//! the engine accepted the move and then pulled them back to the east edge, where they stayed stuck for the whole
//! duel. The ring is now a square centred on the duel (clear of walls where possible), with the exact entry-point
//! return unchanged.
//!
//! Round 146: membership lives in `realm.rs`, and what the ring can really enforce is reported there
//! (`realm::capability`, the stopgap until the native bridge proves each family). The ring no longer pretends to
//! isolate: no attack or cast is rewritten, no hit is healed back afterwards and nobody is made invisible. Duelists
//! only get movement guidance (`guide`): the game's walk, retreat and recall orders become a walk to the opponent.
use crate::contender_core::{map, number, Point, Rect};
use mod_api_stable::{BuffV1, InputKindV1, InputV1, StableSim};
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
/// The ring's centre: the duel's midpoint, kept inside the map, moved up to five cells to the spot with no tower,
/// nexus or camp inside the square and the fewest walls (ties keep the nearest).
pub fn place(entries: [Point; 2], avoid: &[Point]) -> Point {
    let m = map();
    let inside = |c: Point| Point {
        x: c.x.clamp(m.min.x + half(), m.max.x - half()),
        y: c.y.clamp(m.min.y + half(), m.max.y - half()),
    };
    let mid = inside(midpoint(entries));
    let cell = crate::walls::CELL;
    let cost = |c: Point| {
        let r = square(c);
        wall_cells(r) + 50 * avoid.iter().filter(|p| r.contains(**p, -20_000)).count()
    };
    let mut best = (cost(mid), 0i64, mid);
    for dy in -5..=5i64 {
        for dx in -5..=5i64 {
            let c = inside(Point { x: mid.x + dx * cell, y: mid.y + dy * cell });
            let key = (cost(c), dx * dx + dy * dy, c);
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
            r.centre = centres.get(&r.owner).copied().unwrap_or_else(|| place(r.entries, &[]));
            r
        })
        .collect()
}
pub fn member(sim: &StableSim, id: usize) -> Option<Ring> {
    rings(sim)
        .into_iter()
        .find(|r| r.owner == id || r.target == id)
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
/// Towers, the nexus, epic monsters and camps: what the ring must not stand on.
fn fixtures(sim: &StableSim) -> Vec<Point> {
    use mod_api_stable::EntityKindV1 as K;
    (0..sim.entity_count())
        .filter_map(|i| sim.entity_at(i))
        .filter(|e| e.is_alive() && matches!(e.kind(), Some(K::Tower | K::Nexus | K::Epic | K::Serpen | K::Jungle)))
        .map(|e| {
            let (x, y) = e.pos();
            Point { x: x as i64, y: y as i64 }
        })
        .collect()
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
        centre: place(entries, &fixtures(sim)),
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
    if accepted {
        crate::mod_log(
            sim,
            "contender_log.txt",
            &format!("open{}_{}", owner, r.end),
            &format!("Main Event opened: owner {owner} target {target}; {}", crate::realm::capability().report()),
        );
        if let Ok(mut g) = OPENING_HP.lock() {
            let key = crate::match_key(sim);
            g.retain(|o| !(o.0 == key && o.1 == owner));
            if g.len() > 64 {
                g.remove(0);
            }
            g.push((key, owner, [hp(sim, owner), hp(sim, target)]));
        }
    }
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
/// HP each duelist had when the ring opened, per match and ring owner, for the end-of-duel log line.
static OPENING_HP: std::sync::Mutex<Vec<(u64, usize, [usize; 2])>> = std::sync::Mutex::new(Vec::new());
fn hp(sim: &StableSim, id: usize) -> usize {
    sim.get_entity(id).map_or(0, |e| e.hp().0)
}
fn restore(sim: &mut StableSim, r: Ring) -> bool {
    let key = crate::match_key(sim);
    let opened = OPENING_HP.lock().ok().and_then(|mut g| {
        let i = g.iter().position(|o| o.0 == key && o.1 == r.owner)?;
        Some(g.remove(i).2)
    });
    if let Some(h) = opened {
        let lost = [hp(sim, r.owner), hp(sim, r.target)].map(|now| now as i64);
        crate::mod_log(
            sim,
            "contender_log.txt",
            &format!("ring{}_{}", r.owner, r.end),
            &format!(
                "tick {} Main Event ended ({}): owner {} lost {} HP, target {} lost {} HP",
                sim.tick(),
                crate::realm::capability().report(),
                r.owner,
                h[0] as i64 - lost[0],
                r.target,
                h[1] as i64 - lost[1]
            ),
        );
    }
    // summons made in the realm come back beside their owner's return point (or the ring owner's)
    let realm_summons = crate::realm::realms(sim)
        .into_iter()
        .find(|x| x.owner == r.owner)
        .map(|x| x.summons)
        .unwrap_or_default();
    for (n, id) in realm_summons.into_iter().enumerate() {
        if alive(sim, id) {
            let base = r.entries[0];
            let q = map().clamp(Point { x: base.x + 6000 * (n as i64 % 3 - 1), y: base.y + 8000 }, 1000);
            set(sim, id, q);
        }
    }
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
    let open = rings(sim);
    if !open.is_empty() {
        crate::realm::track_summons(sim, &open);
    }
    for r in open {
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
        // lane minions don't walk into the duel: any inside are set just outside the nearest edge
        let b = r.bounds();
        let minions: Vec<(usize, Point)> = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .filter(|e| e.is_alive() && e.is_minion())
            .map(|e| {
                let (x, y) = e.pos();
                (e.id(), Point { x: x as i64, y: y as i64 })
            })
            .filter(|(_, p)| b.contains(*p, -2000))
            .collect();
        for (id, p) in minions {
            let exits = [
                Point { x: b.min.x - 4000, y: p.y },
                Point { x: b.max.x + 4000, y: p.y },
                Point { x: p.x, y: b.min.y - 4000 },
                Point { x: p.x, y: b.max.y + 4000 },
            ];
            if let Some(q) = exits
                .into_iter()
                .filter(|q| map().contains(*q, 1000) && !crate::walls::wall_at(q.x, q.y))
                .min_by_key(|q| q.distance2(p))
            {
                sim.entity_set_pos(id, q.x as u64, q.y as u64);
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

/// Movement guidance for a duelist (None: keep the order). The game's own walk, retreat and recall orders would only
/// press a duelist against the ring's edge, so they become a walk to the opponent. Attacks and casts are never
/// rewritten, and outsiders' orders are never touched.
pub fn guide(sim: &StableSim, me: usize, input: &InputV1) -> Option<InputV1> {
    if input.kind >= InputKindV1::Attack.code() {
        return None;
    }
    let r = rings(sim).into_iter().find(|r| r.owner == me || r.target == me)?;
    let other = if r.owner == me { r.target } else { r.owner };
    let (x, y) = sim.get_entity(other).filter(|e| e.is_alive())?.pos();
    Some(InputV1::move_to(x, y))
}
