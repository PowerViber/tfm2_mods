//! Persistent athlete development. All arithmetic is fixed point and independent of combat RNG.
use std::collections::{BTreeMap, BTreeSet};
#[path = "universal_data.rs"]
pub mod rules;
pub const NAMES: [&str; 4] = ["Macro", "Micro", "Map Awareness", "Strategy"];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratings(pub [i64; 4]);
impl Default for Ratings {
    fn default() -> Self {
        Self([50; 4])
    }
}
fn mix(mut n: u64) -> u64 {
    n ^= n >> 30;
    n = n.wrapping_mul(0xbf58476d1ce4e5b9);
    n ^= n >> 27;
    n = n.wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}
pub fn identity_seed(career: &str, athlete: usize) -> u64 {
    career.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    }) ^ mix(athlete as u64)
}
impl Ratings {
    pub fn initial(v: [i64; 9], seed: u64) -> Self {
        let v = v.map(|n| n.clamp(0, 100));
        let initial = [
            (2 * v[1] + v[8] + v[7] + v[2]) / 5,
            (2 * v[3] + 2 * v[4] + v[5] + v[2]) / 6,
            (2 * v[2] + v[1] + v[8]) / 4,
            (2 * v[1] + 2 * v[7] + v[8]) / 5,
        ];
        Self(std::array::from_fn(|i| {
            (initial[i] + (mix(seed ^ i as u64) % (2 * rules::INITIAL_OFFSET + 1) as u64) as i64
                - rules::INITIAL_OFFSET)
                .clamp(0, 100)
        }))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Development {
    pub ratings: Ratings,
    pub progress: [u16; 4],
    pub focus: usize,
    trained: BTreeSet<u64>,
    sets: BTreeMap<String, u64>,
}
impl Development {
    pub fn new(ratings: Ratings) -> Self {
        let focus = (0..4).min_by_key(|&i| (ratings.0[i], i)).unwrap_or(0);
        Self {
            ratings,
            progress: [0; 4],
            focus,
            trained: BTreeSet::new(),
            sets: BTreeMap::new(),
        }
    }
    fn add(&mut self, k: usize, n: u16) {
        if self.ratings.0[k] >= 100 {
            return;
        }
        let total = self.progress[k] + n;
        self.ratings.0[k] =
            (self.ratings.0[k] + (total / rules::POINTS_PER_RATING as u16) as i64).min(100);
        self.progress[k] = if self.ratings.0[k] == 100 {
            0
        } else {
            total % rules::POINTS_PER_RATING as u16
        };
    }
    /// The caller must establish a completed training record, never an elapsed wall-clock day.
    pub fn training(&mut self, day: u64) -> bool {
        if self.focus >= 4 || !self.trained.insert(day) {
            return false;
        }
        self.add(self.focus, rules::TRAINING_POINTS as u16);
        true
    }
    pub fn learn(&mut self, day: u64, set: &str, evidence: u8) -> bool {
        if evidence & 15 == 0
            || set.is_empty()
            || set.len() > 160
            || set.bytes().any(|b| b.is_ascii_whitespace())
            || self.sets.contains_key(set)
        {
            return false;
        }
        if self.sets.values().filter(|&&d| d == day).count() >= rules::SETS_PER_DAY {
            return false;
        }
        self.sets.insert(set.into(), day);
        for k in 0..4 {
            if evidence & (1 << k) != 0 {
                self.add(k, rules::SET_POINTS as u16);
            }
        }
        true
    }
    pub fn encode(&self) -> String {
        let mut s = format!(
            "1 {} {} {} {} {} {} {} {} {}\n",
            self.focus,
            self.ratings.0[0],
            self.ratings.0[1],
            self.ratings.0[2],
            self.ratings.0[3],
            self.progress[0],
            self.progress[1],
            self.progress[2],
            self.progress[3]
        );
        for d in &self.trained {
            s.push_str(&format!("t {d}\n"));
        }
        for (k, d) in &self.sets {
            s.push_str(&format!("s {d} {k}\n"));
        }
        s
    }
    pub fn decode(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        let v: Vec<i64> = lines
            .next()?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        if v.len() != 10
            || v[0] != 1
            || !(0..4).contains(&v[1])
            || v[2..6].iter().any(|n| !(0..=100).contains(n))
            || v[6..].iter().any(|n| !(0..100).contains(n))
        {
            return None;
        }
        let mut s = Self::new(Ratings(v[2..6].try_into().ok()?));
        s.focus = v[1] as usize;
        s.progress = std::array::from_fn(|i| v[i + 6] as u16);
        for l in lines {
            let p: Vec<_> = l.split_whitespace().collect();
            match p.as_slice() {
                ["t", d] => {
                    s.trained.insert(d.parse().ok()?);
                }
                ["s", d, k] => {
                    s.sets.insert(k.to_string(), d.parse().ok()?);
                }
                _ => return None,
            }
        }
        Some(s)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_ratings_are_stable_bounded_and_individual() {
        let a = Ratings::initial([60; 9], identity_seed("a", 4));
        assert_eq!(a, Ratings::initial([60; 9], identity_seed("a", 4)));
        assert!(a.0.iter().all(|n| (55..=65).contains(n)));
        assert_ne!(a, Ratings::initial([60; 9], identity_seed("a", 5)));
    }
    #[test]
    fn training_and_sets_are_idempotent_and_daily_bounded() {
        let mut d = Development::new(Ratings([50; 4]));
        for day in 0..10 {
            assert!(d.training(day));
            assert!(!d.training(day));
        }
        assert_eq!(d.ratings.0[0], 51);
        for n in 0..4 {
            assert!(d.learn(1, &format!("set{n}"), 15));
        }
        assert!(!d.learn(1, "fifth", 15));
        assert!(!d.learn(2, "set0", 15));
        assert!(d.learn(2, "new", 2));
        assert_eq!(d.progress, [8, 10, 8, 8]);
        assert_eq!(Development::decode(&d.encode()), Some(d));
    }
    #[test]
    fn caps_and_replays_supply_no_free_progress() {
        let mut d = Development::new(Ratings([100; 4]));
        assert!(d.training(1));
        assert_eq!(d.progress, [0; 4]);
        assert!(!d.learn(1, "replay", 0));
        assert!(Development::decode("2 0 50").is_none());
    }
}
