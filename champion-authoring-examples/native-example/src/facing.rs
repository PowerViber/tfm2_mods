//! Facing for champions drawn as buff overlays.
//!
//! The game mirrors only a champion's own body sprite (`champions/<id>#sheet.png`) when it walks left. Buff
//! overlays (`view_buffs`) and view effects are NEVER mirrored. A champion whose visible body is an overlay must
//! ship a mirrored `_l` copy of every body layer (same anchor: the frame centre) and choose it natively.
//! Repo reference: `native/tfm2_custom_ai/src/facing.rs`, used by Vesper and the Unified Theory.
//!
//! The rule is DIO's (`batch2.rs`), the one checked in game: toward the nearest enemy close by while fighting,
//! otherwise the way it walks; standing still keeps the last facing; a short hold stops flicker.

pub const FACE_ENEMY: i64 = 90_000;
pub const FACE_DEAD_ZONE: i64 = 200;
pub const FACE_HOLD: usize = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facing {
    /// 1 right, -1 left (0 until the first update).
    pub face: i8,
    last: Option<(i64, i64)>,
    hold_until: usize,
}

impl Facing {
    /// `enemy`: the nearest enemy's position, if one is within FACE_ENEMY. Call once per update.
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
}

/// The overlay buff name for a body layer: art authors draw `<layer>` facing right and generate `<layer>_l` with
/// `Image.transpose(FLIP_LEFT_RIGHT)`. Rank badges, HUD and anything with text stay unmirrored.
pub fn body_layer(layer: &str, facing: &Facing) -> String {
    if facing.face < 0 { format!("{layer}_l") } else { layer.to_string() }
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
        assert_eq!(f.update(8, (500_000, 0), Some((600_000, 0))), 1);
        assert_eq!(f.update(20, (499_000, 0), None), -1, "walking left, no enemy near");
        assert_eq!(f.update(30, (499_000, 0), None), -1, "standing still keeps it");
        assert_eq!(f.update(31, (499_100, 0), None), -1, "a shuffle under the dead zone does not turn");
    }

    #[test]
    fn left_facing_picks_the_mirrored_layer() {
        let mut f = Facing::default();
        f.update(0, (500_000, 0), Some((400_000, 0)));
        assert_eq!(body_layer("gx_outfit_r3", &f), "gx_outfit_r3_l");
        f.update(10, (500_000, 0), Some((600_000, 0)));
        assert_eq!(body_layer("gx_outfit_r3", &f), "gx_outfit_r3");
    }
}
