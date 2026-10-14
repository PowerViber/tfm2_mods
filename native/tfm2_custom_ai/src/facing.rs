//! Which way a champion whose body is drawn as buff overlays faces. The game mirrors only a champion's own body
//! sprite; buff overlays never flip, so champions drawn in overlays (Vesper, the Unified Theory) ship `_l` copies
//! of their body layers and pick them here.
//!
//! The rule is DIO's (batch2.rs), checked in game: toward the nearest enemy close by while fighting, otherwise the
//! way it moves; standing still keeps the last facing. A short hold stops a jittering walk from flickering.

/// An enemy within this distance decides the facing.
pub const FACE_ENEMY: i64 = 90_000;
/// Movement per update below this does not turn the body.
pub const FACE_DEAD_ZONE: i64 = 200;
/// Ticks a new facing holds before it can turn back.
pub const FACE_HOLD: usize = 6;

#[derive(Clone, Debug, Default)]
pub struct Facing {
    /// 1 right, -1 left (0 until the first update).
    pub face: i8,
    last: Option<(i64, i64)>,
    hold_until: usize,
}

impl Facing {
    /// `enemy`: the nearest enemy's position, if one is within FACE_ENEMY.
    pub fn update(&mut self, tick: usize, me: (i64, i64), enemy: Option<(i64, i64)>) -> i8 {
        if self.face == 0 {
            self.face = 1;
        }
        let want = match enemy.filter(|e| e.0 != me.0) {
            Some(e) => Some(if e.0 > me.0 { 1 } else { -1 }),
            None => self.last.filter(|l| (me.0 - l.0).abs() > FACE_DEAD_ZONE).map(|l| if me.0 > l.0 { 1 } else { -1 }),
        };
        self.last = Some(me);
        if let Some(w) = want {
            if w != self.face && tick >= self.hold_until {
                self.face = w;
                self.hold_until = tick + FACE_HOLD;
            }
        }
        self.face
    }
    pub fn left(&self) -> bool {
        self.face < 0
    }
}

/// The nearest living enemy of `me` within FACE_ENEMY.
pub(crate) fn nearest_enemy(all: &[crate::Champ], me: &crate::Champ) -> Option<(i64, i64)> {
    all.iter()
        .filter(|c| c.team != me.team && c.hp > 0)
        .map(|c| ((c.x - me.x) as i128 * (c.x - me.x) as i128 + (c.y - me.y) as i128 * (c.y - me.y) as i128, c.id, c.x, c.y))
        .filter(|(d2, ..)| *d2 <= (FACE_ENEMY as i128).pow(2))
        .min()
        .map(|(_, _, x, y)| (x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_the_enemy_then_the_walk_and_holds_still() {
        let mut f = Facing::default();
        assert_eq!(f.update(0, (500_000, 0), None), 1, "right by default");
        assert_eq!(f.update(1, (500_000, 0), Some((400_000, 0))), -1, "toward a close enemy on the left");
        assert_eq!(f.update(2, (500_000, 0), Some((600_000, 0))), -1, "held for a few ticks");
        assert_eq!(f.update(8, (500_000, 0), Some((600_000, 0))), 1, "then turns to the enemy on the right");
        assert_eq!(f.update(20, (499_000, 0), None), -1, "walking left with no enemy near");
        assert_eq!(f.update(30, (499_000, 0), None), -1, "standing still keeps the facing");
        assert_eq!(f.update(31, (499_100, 0), None), -1, "a 100-unit shuffle does not turn");
        assert_eq!(f.update(40, (500_000, 0), None), 1, "walking right turns right");
    }
}
