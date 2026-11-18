//! Skill unlocks by champion level, for every native kit (Reihan, round 180): S1 at level 1, S2 at 2, S3 at 3, the ult
//! at 5. Passives are always on. The engine's own slots lock skill2 until 3 and the ult until 5, but native kits cast
//! their skills themselves, so each core asks this rule. A level of 0 means "unknown" (the Labs and unit tests): every
//! skill is unlocked. Dependency-free so the Lab crates can include it.
#![allow(dead_code)]

pub const S1_LEVEL: i64 = 1;
pub const S2_LEVEL: i64 = 2;
pub const S3_LEVEL: i64 = 3;
pub const ULT_LEVEL: i64 = 5;

/// the level a skill slot unlocks at (0 = S1, 1 = S2, 2 = S3; anything higher is the ult)
pub const fn level_for(slot: usize) -> i64 {
    match slot {
        0 => S1_LEVEL,
        1 => S2_LEVEL,
        2 => S3_LEVEL,
        _ => ULT_LEVEL,
    }
}

/// is skill `slot` (0 = S1, 1 = S2, 2 = S3) usable at `level`?
pub const fn skill_unlocked(level: i64, slot: usize) -> bool {
    level <= 0 || level >= level_for(slot)
}

/// the ult: the champion's learned marker (where the kit has one) and level 5
pub const fn ult_unlocked(level: i64, marker: bool) -> bool {
    marker && (level <= 0 || level >= ULT_LEVEL)
}

/// "s2: locked until level 2"
pub fn locked_reason(slot: usize) -> String {
    if slot > 2 {
        format!("ult: locked until level {ULT_LEVEL}")
    } else {
        format!("s{}: locked until level {}", slot + 1, level_for(slot))
    }
}

/// Hold a locked skill's ready tick at least one tick ahead, so the AI and orders see it as on cooldown until the level
/// is reached; once unlocked it is ready at once (its first use is never delayed further).
pub fn hold(ready_at: &mut u64, tick: u64, level: i64, slot: usize) {
    if !skill_unlocked(level, slot) {
        *ready_at = (*ready_at).max(tick + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn s1_at_1_s2_at_2_s3_at_3_and_the_ult_at_5() {
        for (level, s1, s2, s3, ult) in [(1, true, false, false, false), (2, true, true, false, false), (3, true, true, true, false), (4, true, true, true, false), (5, true, true, true, true), (18, true, true, true, true)] {
            assert_eq!([skill_unlocked(level, 0), skill_unlocked(level, 1), skill_unlocked(level, 2), ult_unlocked(level, true)], [s1, s2, s3, ult], "level {level}");
        }
        assert!(!ult_unlocked(18, false), "the marker still matters");
        assert!(skill_unlocked(0, 2) && ult_unlocked(0, true), "unknown level: all unlocked");
        let mut at = 0;
        hold(&mut at, 100, 1, 2);
        assert_eq!(at, 101);
        hold(&mut at, 200, 3, 2);
        assert_eq!(at, 101, "unlocked: untouched");
        assert_eq!(locked_reason(1), "s2: locked until level 2");
    }
}
