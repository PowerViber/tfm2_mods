//! Host-free Omen weapon definitions, also used by Raze and her Lab.
include!("valorant_gun_data.rs");
pub fn judge_percent(distance: i64) -> i64 {
    if distance <= JUDGE_NEAR_RANGE {
        JUDGE_NEAR_PERCENT
    } else if distance >= JUDGE_FAR_RANGE {
        JUDGE_FAR_PERCENT
    } else {
        JUDGE_NEAR_PERCENT
            - (distance - JUDGE_NEAR_RANGE) * (JUDGE_NEAR_PERCENT - JUDGE_FAR_PERCENT)
                / (JUDGE_FAR_RANGE - JUDGE_NEAR_RANGE)
    }
}
pub fn basic_raw(gun: Gun, attack: i64, distance: i64) -> i64 {
    let n = attack * (100 + gun.modifiers().0 as i64) / 100;
    if gun == Gun::Judge {
        n * judge_percent(distance) / 100
    } else {
        n
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gun {
    #[default]
    Classic,
    Sheriff,
    Spectre,
    Judge,
    Vandal,
    Operator,
}
pub fn close_fighter(name: &str) -> bool {
    [
        "fighter",
        "knight",
        "berserker",
        "dual_blader",
        "lancer",
        "monk",
        "ninja",
        "swordman",
        "werewolf",
        "ogre",
        "gladiator",
        "executioner",
        "hammerer",
        "strongman",
        "pole_warrior",
        "steve",
        "vader",
        "dio",
        "v1",
        "david",
    ]
    .iter()
    .any(|n| name.ends_with(n))
}
impl Gun {
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn from_index(n: usize) -> Self {
        [
            Self::Classic,
            Self::Sheriff,
            Self::Spectre,
            Self::Judge,
            Self::Vandal,
            Self::Operator,
        ][n.min(5)]
    }
    pub fn price(self) -> i64 {
        PRICES[self.idx()]
    }
    pub fn tier(self) -> usize {
        TIERS[self.idx()] as usize
    }
    pub fn reach(self) -> i64 {
        RANGES[self.idx()]
    }
    pub fn modifiers(self) -> (i32, i32) {
        (
            ATTACK_MODIFIERS[self.idx()] as i32,
            SPEED_MODIFIERS[self.idx()] as i32,
        )
    }
    pub fn upgrade(credits: i64, close_fighters: usize, current: Self) -> Self {
        let want = if credits >= 5500 {
            Self::Operator
        } else if credits >= 2900 {
            Self::Vandal
        } else if credits >= 1850 && close_fighters >= 2 {
            Self::Judge
        } else if credits >= 1600 {
            Self::Spectre
        } else if credits >= 800 {
            Self::Sheriff
        } else {
            current
        };
        if want.tier() > current.tier() && credits >= want.price() {
            want
        } else {
            current
        }
    }
}
