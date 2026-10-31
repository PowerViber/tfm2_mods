//! Round 146: Main Event realms. A realm belongs to the Contender who opened it and holds the two duelists plus any
//! summon either of them makes while it stands. Membership, not position, separates realms: `can_interact` is the one
//! question every native kit, our planners and farming ask, and several duels at once never meet.
//!
//! What the host can actually enforce is reported honestly (`capability`). Today that is the `stopgap`: the ring,
//! exact returns, duelist movement guidance and Contender's own hits. The game's own AI, turrets, minions and
//! projectiles are NOT isolated until the native bridge proves each family (docs/contender-realms/README.md);
//! every native family flag stays false here.
use crate::contender_arena::Ring;
use crate::contender_core::{Point, Rect};
use mod_api_stable::{EntityKindV1, StableSim};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Realm {
    pub owner: usize,
    pub duelists: [usize; 2],
    /// Summons made by a duelist after the ring opened (existing outside summons stay outside).
    pub summons: Vec<usize>,
    pub bounds: Rect,
}
impl Realm {
    pub fn has(&self, id: usize) -> bool {
        self.duelists.contains(&id) || self.summons.contains(&id)
    }
}

/// The realm `id` belongs to (its owner), or None for the ordinary battlefield.
pub fn realm_of(realms: &[Realm], id: usize) -> Option<usize> {
    realms.iter().find(|r| r.has(id)).map(|r| r.owner)
}
/// Whether `a` and `b` may interact: both in the same realm, or both on the battlefield.
pub fn can_interact(realms: &[Realm], a: usize, b: usize) -> bool {
    realm_of(realms, a) == realm_of(realms, b)
}

/// Per simulation: every summon seen so far (so only new ones can join a realm) and the realm each joined.
#[derive(Default)]
struct Summons {
    seen: BTreeSet<usize>,
    joined: BTreeMap<usize, usize>,
}
static SUMMONS: Mutex<Vec<(u64, Summons)>> = Mutex::new(Vec::new());

/// A summon that first appears inside a realm on a duelist's team joins that realm (the SDK names no summon owner,
/// so a new summon inside the ring with a duelist's team is that duelist's). Pure, so the rule is tested alone.
pub fn assign_new_summon(rings: &[Ring], team_of: impl Fn(usize) -> Option<usize>, team: usize, pos: Point) -> Option<usize> {
    rings
        .iter()
        .find(|r| r.bounds().contains(pos, 0) && [r.owner, r.target].iter().any(|&d| team_of(d) == Some(team)))
        .map(|r| r.owner)
}

/// Note this tick's summons (called once per tick from the match hook, before anyone asks about realms).
pub fn track_summons(sim: &StableSim, rings: &[Ring]) {
    let key = crate::match_key(sim);
    let summons: Vec<(usize, usize, Point)> = (0..sim.entity_count())
        .filter_map(|i| sim.entity_at(i))
        .filter(|e| e.is_alive() && e.kind() == Some(EntityKindV1::Summon))
        .map(|e| {
            let (x, y) = e.pos();
            (e.id(), e.team(), Point { x: x as i64, y: y as i64 })
        })
        .collect();
    let Ok(mut g) = SUMMONS.lock() else { return };
    if !g.iter().any(|s| s.0 == key) {
        if g.len() >= 8 {
            g.remove(0);
        }
        g.push((key, Summons::default()));
    }
    let Some((_, s)) = g.iter_mut().find(|s| s.0 == key) else { return };
    let open: BTreeSet<usize> = rings.iter().map(|r| r.owner).collect();
    s.joined.retain(|_, owner| open.contains(owner));
    let team_of = |id: usize| sim.get_entity(id).map(|e| e.team());
    for (id, team, pos) in summons {
        if s.seen.insert(id) {
            if let Some(owner) = assign_new_summon(rings, team_of, team, pos) {
                s.joined.insert(id, owner);
            }
        }
    }
}

/// Every open realm, with its summons.
pub fn realms(sim: &StableSim) -> Vec<Realm> {
    let rings = crate::contender_arena::rings(sim);
    if rings.is_empty() {
        return vec![];
    }
    let key = crate::match_key(sim);
    let joined: Vec<(usize, usize)> = SUMMONS
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|s| s.0 == key).map(|s| s.1.joined.iter().map(|(a, b)| (*a, *b)).collect()))
        .unwrap_or_default();
    rings
        .iter()
        .map(|r| Realm {
            owner: r.owner,
            duelists: [r.owner, r.target],
            summons: joined.iter().filter(|j| j.1 == r.owner).map(|j| j.0).collect(),
            bounds: r.bounds(),
        })
        .collect()
}

/// Whether `viewer` should take `id` into account at all: visible to its team and in the same realm. Our planners
/// use this so a Main Event duel neither scares nor attracts anyone outside it.
pub fn sees(sim: &StableSim, viewer: usize, team: usize, id: usize) -> bool {
    sim.is_visible(team, id) && {
        let realms = realms(sim);
        realms.is_empty() || can_interact(&realms, viewer, id)
    }
}

/// Which bridge separates realms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// No ring can open (unsupported host).
    None,
    /// The playtest ring: placement, exact returns, duelist movement guidance and Contender's own hits only.
    Stopgap,
    /// Verified engine hooks for every family below.
    Native,
}
/// Each interaction family the native bridge must prove before `Native` (all false until then).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Families {
    pub visibility: bool,
    pub targeting: bool,
    pub damage: bool,
    pub control: bool,
    pub support: bool,
    pub projectiles: bool,
    pub navigation: bool,
    pub collision: bool,
    pub rendering: bool,
}
impl Families {
    pub fn all(self) -> bool {
        self.visibility
            && self.targeting
            && self.damage
            && self.control
            && self.support
            && self.projectiles
            && self.navigation
            && self.collision
            && self.rendering
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability {
    pub backend: Backend,
    pub native: Families,
}
impl Capability {
    /// `Native` only when every family is proven; otherwise the stopgap (or None when nothing can open a ring).
    pub fn from(native: Families, ring_supported: bool) -> Self {
        let backend = if native.all() {
            Backend::Native
        } else if ring_supported {
            Backend::Stopgap
        } else {
            Backend::None
        };
        Capability { backend, native }
    }
    /// One line for contender_log.txt and the Lab.
    pub fn report(self) -> String {
        let n = self.native;
        format!(
            "realm backend {} (native: visibility {} targeting {} damage {} control {} support {} projectiles {} navigation {} collision {} rendering {})",
            match self.backend {
                Backend::None => "none",
                Backend::Stopgap => "stopgap",
                Backend::Native => "native",
            },
            n.visibility,
            n.targeting,
            n.damage,
            n.control,
            n.support,
            n.projectiles,
            n.navigation,
            n.collision,
            n.rendering
        )
    }
}
/// Families the native bridge has proven (set by the bridge after it validates a hook family; none so far).
static NATIVE: Mutex<Families> = Mutex::new(Families {
    visibility: false,
    targeting: false,
    damage: false,
    control: false,
    support: false,
    projectiles: false,
    navigation: false,
    collision: false,
    rendering: false,
});
/// For the native bridge: record the families it has validated on this build.
#[allow(dead_code)]
pub fn set_native(families: Families) {
    if let Ok(mut g) = NATIVE.lock() {
        *g = families;
    }
}
/// What this process can enforce now. The on-map ring always opens (stopgap) until the bridge proves every family.
pub fn capability() -> Capability {
    Capability::from(NATIVE.lock().map(|g| *g).unwrap_or_default(), true)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn realm(owner: usize, target: usize, summons: Vec<usize>) -> Realm {
        Realm {
            owner,
            duelists: [owner, target],
            summons,
            bounds: Rect { min: Point::default(), max: Point { x: 1, y: 1 } },
        }
    }
    #[test]
    fn membership_not_position_separates() {
        let realms = vec![realm(1, 9, vec![20]), realm(3, 4, vec![])];
        assert!(can_interact(&realms, 1, 9));
        assert!(can_interact(&realms, 9, 20), "a summon fights in its owner's realm");
        assert!(!can_interact(&realms, 1, 3), "two duels at once never meet");
        assert!(!can_interact(&realms, 20, 4));
        assert!(!can_interact(&realms, 1, 5), "an outsider is on the battlefield");
        assert!(can_interact(&realms, 5, 6), "outsiders keep each other");
        assert!(can_interact(&[], 1, 2));
    }
    #[test]
    fn only_new_summons_inside_with_a_duelists_team_join() {
        let ring = Ring {
            owner: 1,
            target: 9,
            entries: [Point { x: 400_000, y: 400_000 }, Point { x: 420_000, y: 400_000 }],
            end: 900,
            centre: Point { x: 410_000, y: 400_000 },
        };
        let team = |id: usize| match id {
            1 => Some(0),
            9 => Some(1),
            _ => None,
        };
        let inside = Point { x: 410_000, y: 400_000 };
        let outside = Point { x: 100_000, y: 100_000 };
        assert_eq!(assign_new_summon(&[ring], team, 1, inside), Some(1));
        assert_eq!(assign_new_summon(&[ring], team, 0, inside), Some(1));
        assert_eq!(assign_new_summon(&[ring], team, 2, inside), None, "a third team's summon stays out");
        assert_eq!(assign_new_summon(&[ring], team, 0, outside), None);
    }
    #[test]
    fn capability_is_stopgap_until_every_family_is_proven() {
        let none = Families::default();
        assert_eq!(Capability::from(none, true).backend, Backend::Stopgap);
        assert_eq!(Capability::from(none, false).backend, Backend::None);
        let one = Families { damage: true, ..none };
        assert_eq!(Capability::from(one, true).backend, Backend::Stopgap, "one damage hook is not enough");
        let every = Families {
            visibility: true,
            targeting: true,
            damage: true,
            control: true,
            support: true,
            projectiles: true,
            navigation: true,
            collision: true,
            rendering: true,
        };
        assert_eq!(Capability::from(every, true).backend, Backend::Native);
        let shipped = capability();
        assert_eq!(shipped.backend, Backend::Stopgap);
        assert_eq!(shipped.native, none, "no native family is claimed");
        assert!(shipped.report().starts_with("realm backend stopgap"));
    }
}
