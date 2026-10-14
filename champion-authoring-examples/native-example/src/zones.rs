//! Telling the game's AI that an area is dangerous.
//!
//! There is no engine API for it. The repo's pattern (Omen's smokes `oms:`, Steve's walls `sbw:`, Vesper's rifts and
//! Kidnap fields `vsr:`/`vsk:`) is:
//! 1. the owner PUBLISHES each zone as a buff name on itself, added when the zone opens and removed when it closes
//!    (never re-added every tick), and calls `steve::note_map_buff(seed)` so the input AI's move rules run;
//! 2. `steve::WallAi` (one input AI for every player) PARSES the buff names and rewrites MOVE orders only.
//!    Never rewrite casts or attacks: injected or held casts made the game's planner panic;
//! 3. other mod AIs read them through `batch2::danger_zones`.
//!
//! Encoded buff names are fine for published state like this; don't use them to track a resource your own rules
//! depend on (another mod or the engine can remove buffs).

pub const PREFIX: &str = "gx:";
/// Steer this far outside the edge.
pub const MARGIN: i64 = 8_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub x: i64,
    pub y: i64,
    pub r: i64,
    pub until: usize,
    /// An armed field that is about to strike: enemies standing in it walk out (their dodge).
    pub armed: bool,
}

/// `gx:x:y:r:until:armed`, positions rounded to 4000 so a moving zone only republishes when it really moved.
/// Buff names are capped at 64 bytes (`BUFF_NAME_CAP`), which this format stays well under.
pub fn publish(z: &Zone) -> String {
    format!("{PREFIX}{}:{}:{}:{}:{}", z.x / 4000 * 4000, z.y / 4000 * 4000, z.r, z.until, u8::from(z.armed))
}

pub fn parse(name: &str, tick: usize) -> Option<Zone> {
    let v: Vec<i64> = name.strip_prefix(PREFIX)?.split(':').filter_map(|s| s.parse().ok()).collect();
    (v.len() == 5 && v[3] > tick as i64).then(|| Zone { x: v[0], y: v[1], r: v[2], until: v[3] as usize, armed: v[4] != 0 })
}

fn dist(a: (i64, i64), b: (i64, i64)) -> i64 {
    (((a.0 - b.0) as f64).hypot((a.1 - b.1) as f64)) as i64
}

fn out_of(z: &Zone, from: (i64, i64)) -> (i64, i64) {
    let (dx, dy) = (from.0 - z.x, from.1 - z.y);
    let (dx, dy) = if dx == 0 && dy == 0 { (1, 0) } else { (dx, dy) };
    let n = (dx as f64).hypot(dy as f64);
    let k = (z.r + MARGIN) as f64 / n;
    (z.x + (dx as f64 * k) as i64, z.y + (dy as f64 * k) as i64)
}

/// The move rule for the zone owner's ENEMIES (filter allies out before calling): walk out of an armed field; never
/// walk into any zone; a zone that opened under them is fought in, not fled (fleeing every fresh zone made the
/// owner far too strong in Vesper's fixtures). Returns the replacement move destination, if any.
pub fn steer(zones: &[Zone], me: (i64, i64), dest: (i64, i64)) -> Option<(i64, i64)> {
    let inside = |z: &Zone, p: (i64, i64)| dist(p, (z.x, z.y)) <= z.r;
    if let Some(z) = zones.iter().find(|z| z.armed && inside(z, me)) {
        return Some(out_of(z, me));
    }
    zones.iter().find(|z| inside(z, dest) && !inside(z, me)).map(|z| out_of(z, dest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zone_round_trips_through_its_buff_name() {
        let z = Zone { x: 500_000, y: 480_000, r: 30_000, until: 900, armed: true };
        let name = publish(&z);
        assert!(name.len() <= 64, "{name}");
        assert_eq!(parse(&name, 100), Some(z));
        assert_eq!(parse(&name, 900), None, "expired zones are ignored");
        assert_eq!(parse("oms:1:2:3", 0), None, "other mods' names are not ours");
    }

    #[test]
    fn enemies_never_walk_in_and_walk_out_of_armed_fields() {
        let rift = Zone { x: 500_000, y: 480_000, r: 30_000, until: 900, armed: false };
        let p = steer(&[rift], (400_000, 480_000), (500_000, 480_000)).expect("a move into it is redirected");
        assert!(dist(p, (rift.x, rift.y)) > rift.r);
        assert_eq!(steer(&[rift], (510_000, 480_000), (520_000, 480_000)), None, "opened under them: fought in");
        let field = Zone { armed: true, ..rift };
        let out = steer(&[field], (510_000, 480_000), (500_000, 480_000)).expect("out of an armed field");
        assert!(out.0 > 500_000 + field.r);
        assert_eq!(steer(&[], (0, 0), (1, 1)), None);
    }
}
