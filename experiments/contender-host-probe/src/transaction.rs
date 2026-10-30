use serde::{Deserialize, Serialize};

pub type Point = (u64, u64);
pub const PREFIX: &str = "ctp1:";

/// Only the seven scalar values below become simulation state. No pointers or shared mutable state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pair {
    pub ids: [usize; 2],
    pub entries: [Point; 2],
    pub end_tick: usize,
}

pub trait World {
    fn position(&self, id: usize) -> Option<Point>;
    fn alive(&self, id: usize) -> bool;
    fn teleport(&mut self, id: usize, to: Point) -> bool;
}

impl Pair {
    pub fn capture(world: &impl World, ids: [usize; 2], end_tick: usize) -> Option<Self> {
        if ids[0] == ids[1] || !ids.iter().all(|&id| world.alive(id)) {
            return None;
        }
        let pair = Self {
            ids,
            entries: [world.position(ids[0])?, world.position(ids[1])?],
            end_tick,
        };
        // A ledger must be readable before the first mutation; never capture an off-map return point.
        if pair.entries.iter().any(|p| p.0 > 960_000 || p.1 > 960_000) {
            return None;
        }
        Some(pair)
    }

    pub fn marker(self) -> String {
        format!(
            "{PREFIX}{}:{}:{}:{}:{}:{}:{}",
            self.ids[0],
            self.ids[1],
            self.entries[0].0,
            self.entries[0].1,
            self.entries[1].0,
            self.entries[1].1,
            self.end_tick
        )
    }

    pub fn parse(marker: &str) -> Option<Self> {
        let fields: Vec<_> = marker.strip_prefix(PREFIX)?.split(':').collect();
        if fields.len() != 7 {
            return None;
        }
        let pair = Self {
            ids: [fields[0].parse().ok()?, fields[1].parse().ok()?],
            entries: [
                (fields[2].parse().ok()?, fields[3].parse().ok()?),
                (fields[4].parse().ok()?, fields[5].parse().ok()?),
            ],
            end_tick: fields[6].parse().ok()?,
        };
        // This experiment may only restore captured points on the original battlefield.
        if pair.ids[0] == pair.ids[1] || pair.entries.iter().any(|p| p.0 > 960_000 || p.1 > 960_000)
        {
            return None;
        }
        Some(pair)
    }

    /// Check readback as well as API success. A failed second transfer rolls the first back.
    pub fn enter(self, world: &mut impl World, starts: [Point; 2]) -> bool {
        for (id, point) in self.ids.into_iter().zip(starts) {
            if !world.teleport(id, point) || world.position(id) != Some(point) {
                self.restore(world);
                return false;
            }
        }
        true
    }

    /// Retry failed returns on future ticks; never replace an exact return with a nearby point.
    pub fn restore(self, world: &mut impl World) -> bool {
        let mut restored = true;
        for (id, entry) in self.ids.into_iter().zip(self.entries) {
            if world.alive(id) && world.position(id) != Some(entry) {
                let accepted = world.teleport(id, entry);
                restored &= accepted && world.position(id) == Some(entry);
            }
        }
        restored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    #[derive(Clone)]
    struct Fake {
        positions: BTreeMap<usize, Point>,
        dead: BTreeSet<usize>,
        fail: Option<usize>,
        clamp: bool,
        moves: Vec<(usize, Point)>,
    }
    impl World for Fake {
        fn position(&self, id: usize) -> Option<Point> {
            self.positions.get(&id).copied()
        }
        fn alive(&self, id: usize) -> bool {
            self.positions.contains_key(&id) && !self.dead.contains(&id)
        }
        fn teleport(&mut self, id: usize, p: Point) -> bool {
            self.moves.push((id, p));
            if self.fail == Some(id) {
                return false;
            }
            self.positions.insert(
                id,
                if self.clamp {
                    (p.0.min(960_000), p.1.min(960_000))
                } else {
                    p
                },
            );
            true
        }
    }
    fn fixture() -> Fake {
        Fake {
            positions: [(1, (123_457, 234_569)), (9, (654_321, 765_431))].into(),
            dead: BTreeSet::new(),
            fail: None,
            clamp: false,
            moves: vec![],
        }
    }
    const STARTS: [Point; 2] = [(1_176_000, 240_000), (1_224_000, 240_000)];
    #[test]
    fn exact_integer_return_not_ring_position() {
        let mut w = fixture();
        let before = w.positions.clone();
        let p = Pair::capture(&w, [1, 9], 780).unwrap();
        assert!(p.enter(&mut w, STARTS));
        w.positions.insert(1, (1_200_123, 250_567));
        assert!(p.restore(&mut w));
        assert_eq!(w.positions, before);
    }
    #[test]
    fn rejected_second_transfer_rolls_first_back() {
        let mut w = fixture();
        let before = w.positions.clone();
        let p = Pair::capture(&w, [1, 9], 780).unwrap();
        w.fail = Some(9);
        assert!(!p.enter(&mut w, STARTS));
        assert_eq!(w.positions, before);
    }
    #[test]
    fn successful_api_with_clamped_position_is_failure() {
        let mut w = fixture();
        let before = w.positions.clone();
        let p = Pair::capture(&w, [1, 9], 780).unwrap();
        w.clamp = true;
        assert!(!p.enter(&mut w, STARTS));
        assert_eq!(w.positions, before);
    }
    #[test]
    fn returns_retry_without_losing_the_ledger() {
        let mut w = fixture();
        let p = Pair::capture(&w, [1, 9], 780).unwrap();
        assert!(p.enter(&mut w, STARTS));
        w.fail = Some(1);
        assert!(!p.restore(&mut w));
        w.fail = None;
        assert!(p.restore(&mut w));
        assert_eq!(w.position(1), Some(p.entries[0]));
    }
    #[test]
    fn dead_participant_is_not_teleported_or_revived() {
        let mut w = fixture();
        let p = Pair::capture(&w, [1, 9], 780).unwrap();
        assert!(p.enter(&mut w, STARTS));
        w.dead.insert(1);
        w.moves.clear();
        assert!(p.restore(&mut w));
        assert_eq!(w.moves, vec![(9, p.entries[1])]);
    }
    #[test]
    fn marker_roundtrip_and_corrupt_input() {
        let p = Pair::capture(&fixture(), [1, 9], 780).unwrap();
        assert_eq!(Pair::parse(&p.marker()), Some(p));
        for text in [
            "ctp1:",
            "ctp1:1:1:1:1:1:1:780",
            "ctp1:1:2:99999999:1:1:1:780",
            "ctp1:-1:2:1:1:1:1:780",
            "other:1:2:1:1:1:1:780",
        ] {
            assert_eq!(Pair::parse(text), None);
        }
    }
    #[test]
    fn independent_worlds_and_clones() {
        let mut a = fixture();
        let mut b = a.clone();
        b.positions.insert(1, (888, 999));
        let pa = Pair::capture(&a, [1, 9], 780).unwrap();
        let pb = Pair::capture(&b, [1, 9], 780).unwrap();
        assert!(pa.enter(&mut a, STARTS));
        assert_eq!(b.position(1), Some((888, 999)));
        assert!(pb.enter(&mut b, STARTS));
        assert!(pa.restore(&mut a));
        assert!(pb.restore(&mut b));
        assert_ne!(a.position(1), b.position(1));
    }
    #[test]
    fn no_dead_missing_or_duplicate_participants() {
        let mut w = fixture();
        assert!(Pair::capture(&w, [1, 1], 780).is_none());
        assert!(Pair::capture(&w, [1, 22], 780).is_none());
        w.dead.insert(9);
        assert!(Pair::capture(&w, [1, 9], 780).is_none());
        w.dead.clear();
        w.positions.insert(1, (1_100_000, 20_000));
        assert!(Pair::capture(&w, [1, 9], 780).is_none());
    }
}
