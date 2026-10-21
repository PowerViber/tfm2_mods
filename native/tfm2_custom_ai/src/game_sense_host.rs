//! Management-only evidence bridge and an optional in-game athlete panel.
//! Calendar messages never award progress: a server-observed training change is required.
use mod_api_stable::{
    CommandResultV1, RecordKindV1 as K, StableClient, StableCommand, StableExtension,
    StableServerCtx,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
#[derive(Default)]
struct Training {
    before: BTreeMap<usize, (Value, Value)>,
    day: Option<u64>,
}
static TRAINING: OnceLock<Mutex<Training>> = OnceLock::new();
fn training() -> &'static Mutex<Training> {
    TRAINING.get_or_init(Default::default)
}
pub fn reset(ctx: &StableServerCtx) {
    let mut t = training().lock().unwrap_or_else(|e| e.into_inner());
    *t = Training::default();
    t.day = ctx
        .save_get_string("game_sense_calendar_v1")
        .and_then(|s| s.parse().ok());
    if let Some(p) = PANEL.get() {
        p.lock().unwrap_or_else(|e| e.into_inner()).last_date = None;
    }
}
pub fn day(text: &str) -> Option<u64> {
    let v: Value = serde_json::from_str(text).ok()?;
    let s = v
        .as_str()
        .or_else(|| v.get("date").and_then(Value::as_str))?;
    let d: Vec<u64> = s
        .split(['-', 'T', ' '])
        .take(3)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    if d.len() != 3 || d[0] < 1 || !(1..=12).contains(&d[1]) || !(1..=31).contains(&d[2]) {
        return None;
    }
    Some(d[0] * 10000 + d[1] * 100 + d[2])
}
fn values(v: &Value, out: &mut Vec<f64>) {
    match v {
        Value::Object(m) => {
            for (k, v) in m {
                if k != "version" {
                    values(v, out)
                }
            }
        }
        Value::Array(v) => {
            for v in v {
                values(v, out)
            }
        }
        Value::Number(v) => {
            if let Some(n) = v.as_f64() {
                out.push(n)
            }
        }
        _ => {}
    }
}
pub fn training_changed(
    before: &Value,
    after: &Value,
    stat_before: &Value,
    stat_after: &Value,
) -> bool {
    if !before.is_object() || !after.is_object() {
        return false;
    }
    let (mut a, mut b) = (vec![], vec![]);
    values(before, &mut a);
    values(after, &mut b);
    let (mut sa, mut sb) = (vec![], vec![]);
    values(stat_before, &mut sa);
    values(stat_after, &mut sb);
    a.len() == b.len()
        && !a.is_empty()
        && (a.iter().zip(&b).any(|(a, b)| b > a)
            || (a != b && sa.len() == sb.len() && sa.iter().zip(sb).any(|(a, b)| b > *a)))
}
pub fn before(ctx: &mut StableServerCtx) {
    let mut t = training().lock().unwrap_or_else(|e| e.into_inner());
    t.before.clear();
    for id in ctx.record_ids(K::Athlete) {
        if let (Some(e), Some(s)) = (
            ctx.athlete_get_json(id, "training_exp"),
            ctx.athlete_get_json(id, "stat"),
        ) {
            if let (Ok(e), Ok(s)) = (serde_json::from_str(&e), serde_json::from_str(&s)) {
                t.before.insert(id, (e, s));
            }
        }
    }
}
pub fn after(ctx: &mut StableServerCtx, match_finished: bool) {
    let t = training().lock().unwrap_or_else(|e| e.into_inner());
    let Some(day) = t.day else { return };
    if match_finished {
        return;
    }
    let ids: Vec<_> = t
        .before
        .iter()
        .filter_map(|(id, (e, s))| {
            let after: Value =
                serde_json::from_str(&ctx.athlete_get_json(*id, "training_exp")?).ok()?;
            let stat: Value = serde_json::from_str(&ctx.athlete_get_json(*id, "stat")?).ok()?;
            training_changed(e, &after, s, &stat).then_some(*id)
        })
        .collect();
    drop(t);
    for id in ids {
        crate::athlete_ai::complete_training(ctx, id, day);
    }
}
pub fn command(ctx: &mut StableServerCtx, c: &StableCommand) -> CommandResultV1 {
    match c.command {
        "game_sense_calendar_v1" => {
            if c.sender_player_id.is_some() || c.sender_team_id.is_some() {
                if let Ok(s) = std::str::from_utf8(c.payload) {
                    if let Some(d) = day(s) {
                        training().lock().unwrap_or_else(|e| e.into_inner()).day = Some(d);
                        ctx.save_set_string("game_sense_calendar_v1", &d.to_string());
                    }
                }
            }
            CommandResultV1::Handled
        }
        "game_sense_focus_v1" => {
            if let Ok(s) = std::str::from_utf8(c.payload) {
                let v: Vec<usize> = s
                    .split_whitespace()
                    .filter_map(|v| v.parse().ok())
                    .collect();
                if v.len() == 2 && v[1] < 4 {
                    let own = ctx
                        .athlete_get_i64(v[0], "contract.team")
                        .or_else(|| ctx.athlete_get_i64(v[0], "contract.team_id"))
                        .map(|v| v as usize);
                    let sender = c
                        .sender_team_id
                        .or_else(|| c.sender_player_id.and_then(|p| ctx.player_team_id(p)));
                    if own.is_some() && own == sender {
                        crate::athlete_ai::set_focus(ctx, v[0], v[1]);
                    }
                }
            }
            CommandResultV1::Handled
        }
        _ => CommandResultV1::Pass,
    }
}
#[derive(Default)]
struct PanelState {
    open: bool,
    index: usize,
    ids: Vec<usize>,
    lines: String,
    last_date: Option<(i32, u32, u32)>,
}
static PANEL: OnceLock<Mutex<PanelState>> = OnceLock::new();
pub struct Panel;
impl StableExtension for Panel {
    fn post_update(&self, c: &mut StableClient, _dt: u64) {
        if !c.is_in_game() {
            return;
        }
        let mut p = PANEL
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if c.key_pressed("F8") {
            p.open = !p.open;
        }
        if let Some(d) = c.game_date() {
            if Some(d) != p.last_date {
                let text = format!("\"{:04}-{:02}-{:02}\"", d.0, d.1, d.2);
                if c.send_command("game_sense_calendar_v1", text.as_bytes()) {
                    p.last_date = Some(d);
                }
            }
        }
        if !p.open {
            return;
        }
        p.ids = c.athlete_ids();
        let own_team = c.player_team_id();
        p.ids.sort_by_key(|id| {
            (
                c.record_get_i64(K::Athlete, *id, "contract.team")
                    .or_else(|| c.record_get_i64(K::Athlete, *id, "contract.team_id"))
                    .map(|n| n as usize)
                    != own_team,
                *id,
            )
        });
        if p.ids.is_empty() {
            p.lines = "Athlete records unavailable".into();
            return;
        }
        if c.key_pressed("F9") {
            p.index = (p.index + 1) % p.ids.len();
        }
        p.index %= p.ids.len();
        let id = p.ids[p.index];
        let dev = c
            .save_get_string(&format!("game_sense_v1:{id}"))
            .and_then(|v| crate::game_sense_core::Development::decode(&v));
        if let Some(d) = &dev {
            if c.key_pressed("F10") {
                let text = format!("{id} {}", (d.focus + 1) % 4);
                c.send_command("game_sense_focus_v1", text.as_bytes());
            }
        }
        let name = c
            .athlete_name(id)
            .unwrap_or_else(|| format!("Athlete #{id}"));
        p.lines = format!("{name}\nF8 Close   F9 Next athlete   F10 Training focus\n");
        if let Some(d) = dev {
            for k in 0..4 {
                p.lines.push_str(&format!(
                    "{}: {}  ({}/100 progress){}\n",
                    crate::game_sense_core::NAMES[k],
                    d.ratings.0[k],
                    d.progress[k],
                    if d.focus == k { "  FOCUS" } else { "" }
                ));
            }
        } else {
            p.lines
                .push_str("Profile not yet initialized by management server\n");
        }
        p.lines
            .push_str("Training requires a confirmed server change; results count once.");
    }
    fn post_render(&self, c: &mut StableClient) {
        if !c.is_in_game() || !c.can_draw() {
            return;
        }
        let p = PANEL
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !p.open {
            return;
        }
        c.draw_rect("UI", 30., 90., 650., 300., 990, 8., 0x13232fef);
        for (n, line) in p.lines.lines().enumerate() {
            c.draw_text(
                "UI",
                line,
                "asset/base/font/set/regular",
                (50., 110. + n as f32 * 31., 610., 31.),
                991,
                20.,
                0xe4f2faff,
                mod_api_stable::TextAlignXV1::Left,
                mod_api_stable::TextAlignYV1::Top,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn training_requires_actual_resource_change() {
        let a = serde_json::json!({"version":1,"skill_hit":25.0});
        let b = serde_json::json!({"version":1,"skill_hit":35.0});
        assert!(training_changed(&a, &b, &Value::Null, &Value::Null));
        assert!(!training_changed(&a, &a, &Value::Null, &Value::Null));
        assert!(!training_changed(
            &Value::Null,
            &b,
            &Value::Null,
            &Value::Null
        ));
    }
    #[test]
    fn calendar_is_not_a_reward() {
        assert_eq!(day("\"2026-10-21\""), Some(20261021));
        assert_eq!(day("\"2026-99-21\""), None);
    }
}
