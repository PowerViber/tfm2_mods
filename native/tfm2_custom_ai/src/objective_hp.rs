//! Career rule migration only: no live-simulation HP writes or damage changes.
use mod_api_stable::{
    CommandResultV1, SettingTargetV1, StableCommand, StableServerCtx, StableServerExtension,
};
use serde_json::{json, Value};
pub const KEY: &str = "objective-hp/v1";
const SETTLED: &str = "objective-hp/v1/settled";
const PATHS: [&str; 4] = [
    "/epic_jungle/stat/hp",
    "/epic_jungle/growth/hp",
    "/serpen_jungle/stat/hp",
    "/serpen_jungle/growth/hp",
];
pub trait Store {
    fn settings(&self) -> Option<Value>;
    fn journal(&self) -> Option<String>;
    fn save(&mut self, value: &str) -> bool;
    fn update(&mut self, value: &Value) -> bool;
}
fn values(v: &Value) -> Option<[u64; 4]> {
    let mut n = [0; 4];
    for (i, p) in PATHS.iter().enumerate() {
        n[i] = v.pointer(p)?.as_u64()?;
    }
    (n[0] > 0 && n[2] > 0).then_some(n)
}
fn array(v: &Value, key: &str) -> Option<[u64; 4]> {
    let a = v.get(key)?.as_array()?;
    (a.len() == 4).then_some(())?;
    Some([
        a[0].as_u64()?,
        a[1].as_u64()?,
        a[2].as_u64()?,
        a[3].as_u64()?,
    ])
}
fn persist(s: &mut impl Store, j: &Value) -> bool {
    let raw = j.to_string();
    s.save(&raw) && s.journal().as_deref() == Some(&raw)
}
/// Prepared/restoring journals recover from interruption before or after the atomic settings write.
pub fn migrate(s: &mut impl Store, restore: bool) -> Result<&'static str, &'static str> {
    let mut settings = s.settings().ok_or("GameSetting API unavailable")?;
    let current = values(&settings).ok_or("Invalid objective HP settings")?;
    let mut j = match s.journal() {
        Some(raw) => {
            serde_json::from_str::<Value>(&raw).map_err(|_| "Corrupt objective journal")?
        }
        None if restore => return Ok("No objective migration to restore"),
        None => {
            let mut after = [0; 4];
            for i in 0..4 {
                after[i] = current[i]
                    .checked_mul(2)
                    .filter(|n| *n <= i64::MAX as u64)
                    .ok_or("HP overflow")?;
            }
            let j = json!({"version":1,"phase":"prepared","before":current,"after":after,"originalSettings":settings});
            if !persist(s, &j) {
                return Err("Objective backup could not be verified");
            }
            j
        }
    };
    if j["version"] != 1 {
        return Err("Unsupported objective migration version");
    }
    let before = array(&j, "before").ok_or("Invalid backup")?;
    let after = array(&j, "after").ok_or("Invalid output")?;
    if values(&j["originalSettings"]) != Some(before)
        || (0..4).any(|i| before[i].checked_mul(2) != Some(after[i]))
    {
        return Err("Objective backup/output mismatch");
    }
    let phase = j["phase"].as_str().ok_or("Missing objective phase")?;
    if !matches!(phase, "prepared" | "applied" | "restoring" | "restored") {
        return Err("Invalid objective phase");
    }
    if !restore && matches!(phase, "applied" | "restored") {
        return Ok(if phase == "applied" {
            "Already applied"
        } else {
            "Restored; automatic migration disabled"
        });
    }
    let restoring = restore || phase == "restoring";
    if restoring && phase == "restored" {
        return Ok("Already restored");
    }
    let (from, to, done) = if restoring {
        (after, before, "restored")
    } else {
        (before, after, "applied")
    };
    if current != from && current != to {
        return Err("Objective HP conflict; existing edits preserved");
    }
    if restoring && phase != "restoring" {
        j["phase"] = json!("restoring");
        if !persist(s, &j) {
            return Err("Restore journal could not be verified");
        }
    }
    if current != to {
        for (p, n) in PATHS.iter().zip(to) {
            *settings.pointer_mut(p).ok_or("Missing objective HP path")? = json!(n);
        }
        if !s.update(&settings) || s.settings().and_then(|v| values(&v)) != Some(to) {
            return Err("Objective settings update failed; journal retained");
        }
    }
    j["phase"] = json!(done);
    if !persist(s, &j) {
        return Err("Objective update completed; journal finalization pending");
    }
    Ok(if restoring {
        "Restored objective HP"
    } else {
        "Doubled objective HP and growth"
    })
}
struct Server<'a, 'b>(&'a mut StableServerCtx<'b>);
impl Store for Server<'_, '_> {
    fn settings(&self) -> Option<Value> {
        serde_json::from_str(&self.0.setting_get_json(SettingTargetV1::GameSetting, "")?).ok()
    }
    fn journal(&self) -> Option<String> {
        self.0.save_get_string(KEY)
    }
    fn save(&mut self, v: &str) -> bool {
        self.0.save_set_string(KEY, v)
    }
    fn update(&mut self, v: &Value) -> bool {
        self.0
            .setting_set_json(SettingTargetV1::GameSetting, "", &v.to_string())
    }
}
pub struct ObjectiveRules;
impl ObjectiveRules {
    fn apply_loaded(&self, ctx: &mut StableServerCtx, log_failure: bool) {
        let restore =
            crate::mod_dir().is_some_and(|p| p.join("objective_hp_restore.flag").exists());
        let result = migrate(&mut Server(ctx), restore);
        if result.is_ok() {
            ctx.save_set_string(SETTLED, "1");
        }
        if log_failure || result.is_ok() {
            crate::magnus::BOOK.log(&format!("objective-hp/v1: {result:?}"));
        }
    }
}
impl StableServerExtension for ObjectiveRules {
    fn on_server_start(&self, ctx: &mut StableServerCtx) {
        ctx.save_remove_key(SETTLED);
        self.apply_loaded(ctx, true);
    }
    fn after_management_tick(&self, ctx: &mut StableServerCtx) {
        // Retry hosts whose startup callback precedes loaded-career settings.
        // This never runs inside a match or replay.
        if ctx.save_get_string(SETTLED).as_deref() != Some("1") {
            self.apply_loaded(ctx, false);
        }
    }
    fn handle_command(
        &self,
        ctx: &mut StableServerCtx,
        command: &StableCommand,
    ) -> CommandResultV1 {
        match command.command {
            "objective_hp.inspect" => {
                let payload =
                    json!({"settings":Server(ctx).settings(),"journal":ctx.save_get_string(KEY)})
                        .to_string();
                ctx.emit_event(
                    command.reply_target(),
                    "objective_hp.report",
                    payload.as_bytes(),
                );
                CommandResultV1::Handled
            }
            "objective_hp.restore" => {
                ctx.save_remove_key(SETTLED);
                let result = migrate(&mut Server(ctx), true);
                crate::magnus::BOOK.log(&format!("objective-hp restore: {result:?}"));
                CommandResultV1::Handled
            }
            _ => CommandResultV1::Pass,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_percentage_hp_equations_keep_coefficients_when_objective_hp_doubles(){
        use crate::unified_core::{Core,Fighter};
        let me=Fighter{id:1,team:0,x:100000,y:100000,hp:1000,max_hp:1000,ad:100,ap:200,champion:true,visible:true,targetable:true,cc_immune:false};
        let mut boss=Fighter{id:99,team:2,x:160000,y:100000,hp:10000,max_hp:10000,champion:false,..me.clone()};
        let core=Core::new(1,1,7);let mut rows=vec![];
        for (id,e) in crate::unified_eq_data::EQS.iter().enumerate().filter(|(_,e)|e.hp>0){
            boss.hp=10000;boss.max_hp=10000;let before=core.raw(id,&me,Some(&boss),100,100);let fixed=core.raw(id,&me,None,100,100);
            boss.hp=20000;boss.max_hp=20000;let after=core.raw(id,&me,Some(&boss),100,100);assert_eq!(after-fixed,2*(before-fixed));
            rows.push(json!({"equation":e.name,"hpPercent":e.hp,"rawBefore":before,"rawAfter":after,"fixedPart":fixed}));
        }
        assert!(!rows.is_empty());
        if let Ok(path)=std::env::var("OBJECTIVE_PERCENT_REPORT"){std::fs::write(path,serde_json::to_string_pretty(&rows).unwrap()).unwrap();}
    }
    struct Memory {
        settings: Value,
        journal: Option<String>,
        writes: usize,
        fail_save: bool,
        fail_update: bool,
    }
    impl Store for Memory {
        fn settings(&self) -> Option<Value> {
            Some(self.settings.clone())
        }
        fn journal(&self) -> Option<String> {
            self.journal.clone()
        }
        fn save(&mut self, v: &str) -> bool {
            if self.fail_save {
                return false;
            }
            self.journal = Some(v.into());
            true
        }
        fn update(&mut self, v: &Value) -> bool {
            if self.fail_update {
                return false;
            }
            self.settings = v.clone();
            self.writes += 1;
            true
        }
    }
    fn memory() -> Memory {
        Memory {
            settings: json!({"epic_jungle":{"stat":{"hp":10000,"attack":150},"growth":{"hp":1500},"gold":200},"serpen_jungle":{"stat":{"hp":5000},"growth":{"hp":1000},"respawn_tick":7200},"career":"preserved"}),
            journal: None,
            writes: 0,
            fail_save: false,
            fail_update: false,
        }
    }
    #[test]
    fn doubles_only_four_fields_and_never_repeats() {
        let mut m = memory();
        let original = m.settings.clone();
        migrate(&mut m, false).unwrap();
        assert_eq!(values(&m.settings), Some([20000, 3000, 10000, 2000]));
        for _ in 0..4 {
            migrate(&mut m, false).unwrap();
        }
        assert_eq!(m.writes, 1);
        migrate(&mut m, true).unwrap();
        assert_eq!(m.settings, original);
        migrate(&mut m, false).unwrap();
        assert_eq!(m.settings, original);
    }
    #[test]
    fn custom_growth_and_unrelated_changes_survive_restore() {
        let mut m = memory();
        m.settings["epic_jungle"]["stat"]["hp"] = json!(12345);
        migrate(&mut m, false).unwrap();
        m.settings["career"] = json!("later edit");
        migrate(&mut m, true).unwrap();
        assert_eq!(values(&m.settings).unwrap()[0], 12345);
        assert_eq!(m.settings["career"], "later edit");
    }
    #[test]
    fn interrupted_settings_or_finalization_recovers_without_double_scaling() {
        let mut m = memory();
        m.fail_update = true;
        assert!(migrate(&mut m, false).is_err());
        m.fail_update = false;
        migrate(&mut m, false).unwrap();
        let mut j: Value = serde_json::from_str(m.journal.as_ref().unwrap()).unwrap();
        j["phase"] = json!("prepared");
        m.journal = Some(j.to_string());
        migrate(&mut m, false).unwrap();
        assert_eq!(values(&m.settings), Some([20000, 3000, 10000, 2000]));
        assert_eq!(m.writes, 1);
    }
    #[test]
    fn conflicts_and_failed_backups_never_change_settings() {
        let mut m = memory();
        m.fail_save = true;
        let old = m.settings.clone();
        assert!(migrate(&mut m, false).is_err());
        assert_eq!(m.settings, old);
        m.fail_save = false;
        migrate(&mut m, false).unwrap();
        m.settings["serpen_jungle"]["stat"]["hp"] = json!(9999);
        let edited = m.settings.clone();
        assert!(migrate(&mut m, true).is_err());
        assert_eq!(m.settings, edited);
    }
    #[test]
    fn interrupted_restore_finishes_and_stays_restored() {
        let mut m = memory();
        migrate(&mut m, false).unwrap();
        let mut j: Value = serde_json::from_str(m.journal.as_ref().unwrap()).unwrap();
        j["phase"] = json!("restoring");
        m.journal = Some(j.to_string());
        migrate(&mut m, false).unwrap();
        assert_eq!(values(&m.settings), Some([10000, 1500, 5000, 1000]));
        migrate(&mut m, false).unwrap();
        assert_eq!(m.writes, 2);
    }
}
