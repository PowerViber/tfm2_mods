//! Grimhald's pentagon (Warden's Verdict), enforced every tick by the match hook.
//!
//! State lives in buffs, so both simulations of a match agree and nothing hides in native memory:
//!   `grh_cage:<cx>:<cy>:<r>:<until>` on Grimhald while his cage stands;
//!   `grh_in:<owner>` on every champion caged by it (Grimhald too), for as long as it stands.
//! The five edges are also `sbw:` walls on Grimhald, so the shared wall code already makes everyone path round them,
//! stops walking, dashes and knockbacks at them, keeps blinks from landing in them, lets Levi's cables bite and
//! blocks sight across them (`steve::wall_fog`, or `valorant::fog` while a smoke is up): inside sees only inside,
//! outside never sees in, as with Omen's smoke.
//!
//! What this module adds on top:
//!   * blinks, Flash and teleports (which go over walls) can't take a caged champion out: back inside next tick;
//!   * nobody gets in: an uncaged champion inside (or in the wall) is put outside;
//!   * minions, camps and summons are put outside (minions 30,000 past the edge, out of their reach), so they can't
//!     attack the caged champions, and the caged can't reach them; towers never stand in a cage (the ult planner
//!     keeps it out of tower range).
//!   * `can_interact`: native kits (Grimhald's own hits first) never cross the wall either way.
//! Honest limit: a stock ranged champion inside could still shoot a unit outside if one stood in reach; nothing
//! stands in reach, but the engine has no hook to forbid it.
use crate::grimhald_core::{self as core, Cage, Point};
use crate::Champ;
use mod_api_stable::StableSim;

pub const CAGE_PREFIX: &str = "grh_cage:";
pub const MEMBER_PREFIX: &str = "grh_in:";
/// how far inside the edge a caged champion is put back
const IN_MARGIN: i64 = core::WALL_HALF + 3_000;
/// how far outside the edge an outsider is put
const OUT_MARGIN: i64 = core::WALL_HALF + 3_000;
/// minions go further: out of their attack reach
const MINION_MARGIN: i64 = 30_000;
/// on Grimhald while he leaps to the axe
pub const LEAP_BUFF: &str = "grh_leap";

pub fn cage_buff(c: &Cage) -> String {
    format!("{CAGE_PREFIX}{}:{}:{}:{}", c.center.x, c.center.y, c.radius, c.until)
}
pub fn member_buff(owner: usize) -> String {
    format!("{MEMBER_PREFIX}{owner}")
}

pub fn parse_cage(owner: usize, name: &str) -> Option<Cage> {
    let v: Vec<i64> = name.strip_prefix(CAGE_PREFIX)?.split(':').filter_map(|s| s.parse().ok()).collect();
    (v.len() == 4).then(|| Cage { owner, center: Point::new(v[0], v[1]), radius: v[2], until: v[3].max(0) as u64 })
}

pub fn cages(all: &[Champ], tick: usize) -> Vec<Cage> {
    let mut out = vec![];
    for c in all {
        for b in &c.buffs {
            if let Some(cage) = parse_cage(c.id, b.name()) {
                if (tick as u64) < cage.until {
                    out.push(cage);
                }
            }
        }
    }
    out
}

pub fn member_of(c: &Champ) -> Option<usize> {
    c.buffs.iter().find_map(|b| b.name().strip_prefix(MEMBER_PREFIX).and_then(|s| s.parse().ok()))
}

/// Can `a` and `b` affect each other? Both in the same cage, or both outside every cage. Only champions are
/// members; a non-champion is always outside (and is kept outside).
pub fn can_interact(all: &[Champ], a: usize, b: usize) -> bool {
    let m = |id: usize| all.iter().find(|c| c.id == id).and_then(member_of);
    m(a) == m(b)
}

/// The match hook: hold members in, keep everyone else out.
pub fn tick(sim: &mut StableSim<'_>, all: &[Champ], tick: usize) {
    // cheap early out: no cage buff anywhere
    if !all.iter().any(|c| c.buffs.iter().any(|b| b.name().starts_with(CAGE_PREFIX))) {
        return;
    }
    let cages = cages(all, tick);
    if cages.is_empty() {
        return;
    }
    let mut moved: Vec<(usize, Point)> = vec![];
    for cage in &cages {
        // still rising: the edge isn't solid yet
        for c in all {
            let p = Point::new(c.x, c.y);
            match member_of(c) {
                Some(owner) if owner == cage.owner => {
                    // Grimhald mid-leap is flying in: leave him to the leap
                    if c.has(LEAP_BUFF) {
                        continue;
                    }
                    if !core::inside(cage.center, cage.radius, p, IN_MARGIN / 2) {
                        moved.push((c.id, core::clamp_inside(cage.center, cage.radius, p, IN_MARGIN)));
                    }
                }
                _ => {
                    if core::depth(cage.center, cage.radius, p) > -(OUT_MARGIN as f64) / 2.0 {
                        moved.push((c.id, core::push_outside(cage.center, cage.radius, p, OUT_MARGIN)));
                    }
                }
            }
        }
        // minions, camps and summons: out, and minions out of reach
        let near = (cage.radius + MINION_MARGIN + 10_000) as i128;
        for i in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(i) else { continue };
            if !e.is_alive() || e.is_champion() || e.is_tower() {
                continue;
            }
            let (x, y) = e.pos();
            let p = Point::new(x as i64, y as i64);
            if p.d2(cage.center) > near * near {
                continue;
            }
            let margin = if e.is_minion() { MINION_MARGIN } else { OUT_MARGIN };
            if core::depth(cage.center, cage.radius, p) > -(margin as f64) {
                moved.push((e.id(), core::push_outside(cage.center, cage.radius, p, margin)));
            }
        }
    }
    for (id, p) in moved {
        // onto open ground (terrain or another wall in the way: the nearest free point on the line)
        let from = all.iter().find(|c| c.id == id).map(|c| (c.x, c.y)).unwrap_or((p.x, p.y));
        let (x, y) = crate::place::safe(all, tick, from, (p.x, p.y));
        sim.entity_set_pos(id, x.max(0) as u64, y.max(0) as u64);
        crate::steve::note_placed(sim, id, x, y);
        if all.iter().any(|c| c.id == id) {
            sim.add_buff(id, &crate::timed(crate::place::MOVED, 2));
        }
    }
}

/// When the cage rises: every champion inside it (and Grimhald, who is leaping in) is caged until it falls.
pub fn mark_members(sim: &mut StableSim<'_>, all: &[Champ], cage: &Cage, tick: usize) -> Vec<usize> {
    let ticks = cage.until.saturating_sub(tick as u64) as usize + 1;
    let tag = member_buff(cage.owner);
    let mut ids = vec![];
    for c in all {
        let inside = core::inside(cage.center, cage.radius, Point::new(c.x, c.y), 0);
        if (inside || c.id == cage.owner) && member_of(c).is_none() {
            sim.add_buff(c.id, &crate::timed(&tag, ticks));
            ids.push(c.id);
        }
    }
    ids
}

/// The cage falls (or its owner died): every mark goes.
pub fn release(sim: &mut StableSim<'_>, all: &[Champ], owner: usize) {
    let tag = member_buff(owner);
    for c in all.iter().filter(|c| c.has(&tag)) {
        sim.entity_remove_buff(c.id, &tag);
    }
    if let Some(c) = all.iter().find(|c| c.id == owner) {
        for b in c.buffs.iter().filter(|b| b.name().starts_with(CAGE_PREFIX)) {
            sim.entity_remove_buff(owner, b.name());
        }
    }
}
