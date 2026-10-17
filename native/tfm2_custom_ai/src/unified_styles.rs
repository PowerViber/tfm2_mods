//! Career-owned preferences. Management persists them; simulations only read pinned copies.
pub use crate::rework_data::STYLE_NAMES as NAMES;
use mod_api_stable::{RecordKindV1, StableServerCtx, StableServerExtension};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub style: usize,
    pub signatures: [usize; 2],
}
fn mix(mut n: u64) -> u64 {
    n ^= n >> 30;
    n = n.wrapping_mul(0xbf58476d1ce4e5b9);
    n ^= n >> 27;
    n = n.wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}
impl Profile {
    pub fn generate(career: u64, athlete: usize) -> Self {
        let h = mix(career ^ (athlete as u64).wrapping_mul(0x9e3779b97f4a7c15));
        let style = (h % 6) as usize;
        let a = ((h >> 8) % 50) as usize;
        let b = (a + 1 + ((h >> 24) % 49) as usize) % 50;
        Self {
            style,
            signatures: [a, b],
        }
    }
    pub fn encode(self) -> String {
        format!(
            "1 {} {} {}",
            self.style, self.signatures[0], self.signatures[1]
        )
    }
    pub fn parse(s: &str) -> Option<Self> {
        let n: Vec<usize> = s
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        if n.len() != 4 || n[0] != 1 || n[1] >= 6 || n[2] >= 50 || n[3] >= 50 || n[2] == n[3] {
            return None;
        }
        Some(Self {
            style: n[1],
            signatures: [n[2], n[3]],
        })
    }
    /// 0..100 preference points; caller scales to at most 10% inside one tactical band.
    pub fn preference(self, template: usize, sciences: u8, protect: bool, length: usize) -> i64 {
        let signature = template >= 75 && self.signatures.contains(&(template - 75));
        let preferred = match self.style {
            0 => sciences & 1 != 0,
            1 => sciences & 2 != 0,
            2 => sciences & 4 != 0,
            3 => protect,
            4 => length <= 2,
            _ => sciences == 7 && length >= 3,
        };
        (if preferred {
            if self.style < 3 {
                60 / sciences.count_ones().max(1) as i64
            } else {
                60
            }
        } else {
            0
        }) + (if signature { 40 } else { 0 })
    }
}
#[derive(Default)]
struct Session {
    career: u64,
    profiles: BTreeMap<usize, Profile>,
    pins: BTreeMap<(u64, usize), Profile>,
}
static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
pub fn pinned(seed: u64, athlete: Option<usize>) -> Profile {
    let id = athlete.unwrap_or(usize::MAX);
    let Ok(mut s) = SESSION
        .get_or_init(|| Mutex::new(Session::default()))
        .lock()
    else {
        return Profile::generate(0, id);
    };
    if let Some(p) = s.pins.get(&(seed, id)) {
        return *p;
    }
    let p = s
        .profiles
        .get(&id)
        .copied()
        .unwrap_or_else(|| Profile::generate(s.career, id));
    if s.pins.len() >= 4096 {
        s.pins.clear();
    }
    s.pins.insert((seed, id), p);
    p
}
pub struct StyleServer;
impl StyleServer {
    fn load(&self, ctx: &mut StableServerCtx) {
        let key = "unified/career-v1";
        let career = ctx
            .save_get_string(key)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                // Management-only namespace creation; never sampled by a combat callback.
                let n = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(1, |t| t.as_nanos() as u64);
                let id = mix(n);
                if ctx.save_set_string(key, &id.to_string()) {
                    id
                } else {
                    0
                }
            });
        let ids = ctx.record_ids(RecordKindV1::Athlete);
        if SESSION.get().and_then(|s| s.lock().ok()).is_some_and(|s| {
            s.career == career
                && s.profiles.len() == ids.len()
                && ids.iter().all(|id| s.profiles.contains_key(id))
        }) {
            return;
        }
        let mut profiles = BTreeMap::new();
        for athlete in ids {
            let key = format!("unified/styles/v1/{athlete}");
            let p = ctx
                .save_get_string(&key)
                .as_deref()
                .and_then(Profile::parse)
                .unwrap_or_else(|| {
                    let p = Profile::generate(career, athlete);
                    ctx.save_set_string(&key, &p.encode());
                    p
                });
            profiles.insert(athlete, p);
        }
        if let Ok(mut s) = SESSION
            .get_or_init(|| Mutex::new(Session::default()))
            .lock()
        {
            if s.career != career {
                s.pins.clear();
            }
            s.career = career;
            s.profiles = profiles;
        }
    }
}
impl StableServerExtension for StyleServer {
    fn on_server_start(&self, ctx: &mut StableServerCtx) {
        self.load(ctx);
    }
    fn after_management_tick(&self, ctx: &mut StableServerCtx) {
        self.load(ctx);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identities_are_stable_distinct_and_serializable() {
        let a = Profile::generate(42, 7);
        assert_eq!(a, Profile::generate(42, 7));
        assert_eq!(Profile::parse(&a.encode()), Some(a));
        assert_ne!(a.signatures[0], a.signatures[1]);
        assert!(Profile::parse("1 6 0 1").is_none());
        let values: std::collections::BTreeSet<_> =
            (0..100).map(|i| Profile::generate(42, i).style).collect();
        assert_eq!(values.len(), 6);
    }
    #[test]
    fn preferences_are_bounded_and_do_not_lock_spells() {
        for style in 0..6 {
            let p = Profile {
                style,
                signatures: [1, 2],
            };
            for id in 0..125 {
                for mask in 1..8 {
                    assert!((0..=100).contains(&p.preference(id, mask, false, 5)));
                }
            }
        }
    }
}
