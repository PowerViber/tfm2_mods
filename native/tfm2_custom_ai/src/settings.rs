//! Player-editable switches in `mods/tfm2_custom_ai/settings.json`, read once per process (and again at each
//! server start). A missing or unreadable file means every switch is on.
//!
//! ```json
//! { "assist": { "athlete_moves": true, "team_guidance": true, "team_strategy": true } }
//! ```
//! Turning a switch off hands that layer back to the engine, so a match can show which layer causes an oddity:
//! - `athlete_moves`: the athlete's profile values reshaping move orders (athlete_play.rs);
//! - `team_guidance`: the team planner walking players toward their job points (universal_ai::movement);
//! - `team_strategy`: the team planner writing the engine's team strategy fields (Gather/Split, Wave/Join...).
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assist {
    pub athlete_moves: bool,
    pub team_guidance: bool,
    pub team_strategy: bool,
}

impl Default for Assist {
    fn default() -> Self {
        Assist { athlete_moves: true, team_guidance: true, team_strategy: true }
    }
}

static ASSIST: Mutex<Option<Assist>> = Mutex::new(None);

/// The switches from a settings document; anything missing stays on.
pub fn parse(text: &str) -> Assist {
    let mut out = Assist::default();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(text) else { return out };
    let a = &v["assist"];
    let flag = |k: &str, d: bool| a.get(k).and_then(|x| x.as_bool()).unwrap_or(d);
    out.athlete_moves = flag("athlete_moves", out.athlete_moves);
    out.team_guidance = flag("team_guidance", out.team_guidance);
    out.team_strategy = flag("team_strategy", out.team_strategy);
    out
}

fn load() -> Assist {
    crate::mod_dir()
        .and_then(|d| std::fs::read_to_string(d.join("settings.json")).ok())
        .map_or_else(Assist::default, |t| parse(&t))
}

/// The current switches (read from disk the first time).
pub fn assist() -> Assist {
    let Ok(mut g) = ASSIST.lock() else { return Assist::default() };
    *g.get_or_insert_with(load)
}

/// Re-read the file (server start), so an edit takes effect without restarting the game.
pub fn reload() {
    if let Ok(mut g) = ASSIST.lock() {
        *g = Some(load());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_bad_settings_keep_every_switch_on() {
        assert_eq!(parse(""), Assist::default());
        assert_eq!(parse("{}"), Assist::default());
        assert_eq!(parse("{\"assist\":{\"team_guidance\":\"no\"}}"), Assist::default());
    }
    #[test]
    fn each_switch_turns_off_alone() {
        let a = parse("{\"assist\":{\"team_strategy\":false}}");
        assert!(a.athlete_moves && a.team_guidance && !a.team_strategy);
        let a = parse("{\"assist\":{\"athlete_moves\":false,\"team_guidance\":false}}");
        assert!(!a.athlete_moves && !a.team_guidance && a.team_strategy);
    }
}
