//! Round 156: moves that should be *seen* moving go through the engine, never `entity_set_pos`.
//!
//! `entity_set_pos` only writes a unit's coordinates. The game never learns the unit is moving, so the client draws a
//! jump ("it teleports instead of dashing") and the unit keeps its standing pose. Base-game dashes and knockbacks are
//! smooth because the engine moves the unit itself. Here the same happens through the engine's forced movement (a
//! `ForceMove` CC: a direction and a speed), one step per tick:
//!   * `step(sim, id, to, kind)` — move `id` to `to` over the next tick (dashes, flights, knockback paths that the
//!     kits still plan tick by tick, so their wall checks stay exact);
//!   * `glide(sim, id, to, ticks, kind)` — one command for a whole straight push (a satchel blast).
//! `entity_set_pos` stays right for real teleports (Flash, Minato, Omen, Steve's pearl), for invisible units, and for
//! corrective snaps (cage and domain edges). `motion_guard_tests.rs` fails on any new `entity_set_pos` call that is
//! not on its allowlist.
//!
//! Every motion CC is named `mv_self` (a champion moving himself) or `mv_knock` (a unit being pushed), so kits can
//! tell their own dash from real crowd control (`is_self_motion`).
//!
//! `motion.flag` next to the DLL brings back the old hand-set positions (an A/B switch, read once per game launch);
//! `motion_trace.flag` writes every motion to `motion_log.txt`.
use mod_api_stable::{CcKindV1, CcV1, StableSim};

pub const SELF: &str = "mv_self";
pub const KNOCK: &str = "mv_knock";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// a champion moving himself (dash, leap, flight): his own checks must not read it as control
    SelfMove,
    /// a unit pushed by someone else: it *is* control for everyone
    Knock,
}
impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::SelfMove => SELF,
            Kind::Knock => KNOCK,
        }
    }
}

fn flag(name: &str) -> bool {
    crate::mod_dir().is_some_and(|d| d.join(name).exists())
}

#[cfg(not(test))]
pub fn legacy() -> bool {
    static LEGACY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LEGACY.get_or_init(|| flag("motion.flag"))
}
#[cfg(not(test))]
fn tracing() -> bool {
    static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TRACE.get_or_init(|| flag("motion_trace.flag"))
}
#[cfg(test)]
thread_local! {
    static TEST_LEGACY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
#[cfg(test)]
pub fn legacy() -> bool {
    TEST_LEGACY.with(|c| c.get())
}
#[cfg(test)]
pub(crate) fn set_legacy_for_test(on: bool) {
    TEST_LEGACY.with(|c| c.set(on));
}
#[cfg(test)]
fn tracing() -> bool {
    let _ = flag;
    false
}

/// Is this CC a champion's own dash or flight (not control)?
pub fn is_self_motion(c: &CcV1) -> bool {
    c.kind == CcKindV1::ForceMove.code() && c.name() == SELF
}

fn pos(sim: &StableSim<'_>, id: usize) -> Option<(i64, i64)> {
    sim.get_entity(id).filter(|e| e.is_alive()).map(|e| {
        let p = e.pos();
        (p.0 as i64, p.1 as i64)
    })
}

fn force(sim: &mut StableSim<'_>, id: usize, from: (i64, i64), to: (i64, i64), ticks: u64, kind: Kind) -> bool {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = ((dx as f64).hypot(dy as f64)).round() as i64;
    if len <= 0 {
        return false;
    }
    // other systems (Steve's wall check) leave a deliberately moved unit alone for this tick
    sim.add_buff(id, &crate::timed(crate::place::MOVED, ticks as usize + 1));
    if legacy() {
        sim.entity_set_pos(id, to.0.max(0) as u64, to.1.max(0) as u64);
        crate::steve::note_placed(sim, id, to.0, to.1);
    } else {
        let mut c = CcV1::of_kind(CcKindV1::ForceMove, ticks.max(1));
        c.dx = dx * 1000 / len;
        c.dy = dy * 1000 / len;
        c.speed = (len as u64).div_ceil(ticks.max(1));
        c.set_name(kind.name());
        sim.apply_cc(id, &c);
    }
    if tracing() {
        crate::mod_log(
            sim,
            "motion_log.txt",
            &format!("{}:{id}", sim.tick()),
            &format!("{} {id} {from:?} -> {to:?} over {ticks} ({})", kind.name(), if legacy() { "set_pos" } else { "engine" }),
        );
    }
    true
}

/// Move `id` to `to` over the next tick. Returns false when there is nothing to do (dead, or already there).
pub fn step(sim: &mut StableSim<'_>, id: usize, to: (i64, i64), kind: Kind) -> bool {
    let Some(from) = pos(sim, id) else { return false };
    force(sim, id, from, to, 1, kind)
}

/// Push `id` in a straight line to `to` over `ticks` (one engine command; in legacy mode it lands at once).
pub fn glide(sim: &mut StableSim<'_>, id: usize, to: (i64, i64), ticks: u64, kind: Kind) -> bool {
    let Some(from) = pos(sim, id) else { return false };
    force(sim, id, from, to, ticks, kind)
}
