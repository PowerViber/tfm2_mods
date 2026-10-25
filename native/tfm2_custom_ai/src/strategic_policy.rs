//! Translate intentions to supported strategy fields; engine-owned actions remain untouched.
use crate::strategic_core::Goal;
pub fn settings(goal: Goal) -> &'static [(&'static str, &'static str)] {
    match goal {
        Goal::Wave | Goal::Trade | Goal::Push => {
            &[("minion_wave", "WavePriority"), ("object_buildup", "Split")]
        }
        // round 142: a camp plan leaves the wave priority to the coach (WavePriority could route the jungler to waves)
        Goal::Camp => &[("object_buildup", "Split")],
        Goal::Prepare | Goal::Reinforce => &[
            ("object_buildup", "Gather"),
            ("minion_wave", "WavePriority"),
        ],
        Goal::Start | Goal::Contest => &[
            ("object_buildup", "Gather"),
            ("object_battle", "Initiating"),
            ("object_finish", "BattlePriority"),
            ("minion_wave", "JoinPriority"),
        ],
        Goal::Finish => &[
            ("object_buildup", "Gather"),
            ("object_finish", "KillPriority"),
            ("minion_wave", "JoinPriority"),
        ],
        Goal::Stall => &[
            ("object_battle", "Poking"),
            ("object_finish", "BattlePriority"),
            ("minion_wave", "JoinPriority"),
        ],
        Goal::Defend => &[("minion_wave", "JoinPriority")],
        Goal::Withdraw | Goal::Reset => &[
            ("object_buildup", "Split"),
            ("object_battle", "Poking"),
            ("minion_wave", "WavePriority"),
        ],
    }
}
pub fn known(path: &str, value: &str) -> bool {
    match path {
        "minion_wave" => matches!(value, "WavePriority" | "JoinPriority"),
        "object_buildup" => matches!(value, "Gather" | "Split"),
        "object_battle" => matches!(value, "Poking" | "Initiating"),
        "object_finish" => matches!(value, "KillPriority" | "BattlePriority"),
        _ => false,
    }
}
pub fn priority(goal: Goal, wave_first: bool, gather: bool, finish: bool) -> i64 {
    i64::from(wave_first && matches!(goal, Goal::Wave | Goal::Defend)) * 12
        + i64::from(
            gather
                && matches!(
                    goal,
                    Goal::Prepare | Goal::Start | Goal::Contest | Goal::Stall
                ),
        ) * 10
        + i64::from(finish && goal == Goal::Finish) * 12
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_known_fields_are_emitted() {
        for code in 0..=12 {
            for (p, v) in settings(Goal::from_code(code).unwrap()) {
                assert!(known(p, v));
            }
        }
        assert!(!known("focused", "All"));
    }
    #[test]
    fn coach_priorities_influence_close_choices() {
        assert!(
            priority(Goal::Wave, true, false, false) > priority(Goal::Contest, true, false, false)
        );
    }
}

#[cfg(test)]
mod camp_policy_tests {
    use super::*;
    #[test]
    fn a_camp_plan_leaves_the_wave_priority_to_the_coach() {
        assert!(settings(Goal::Camp).iter().all(|(f, _)| *f != "minion_wave"));
        assert!(settings(Goal::Wave).iter().any(|(f, v)| *f == "minion_wave" && *v == "WavePriority"));
    }
}
