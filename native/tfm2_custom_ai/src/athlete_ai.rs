//! Supported management reads -> immutable match profiles. No private game memory or athlete writes.
use crate::athlete_ai_core::{Actor, Objective, Profile, Scene, PROFILE_FIELDS};
use mod_api_stable::{StableClient, StableServerCtx, StableServerExtension, StableSim};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Mutex, OnceLock};
/// Pin keys are derived from the simulation alone (match, set, seed, athlete), so presim,
/// match view and replay find the same pin in any process. v1 keys also carried the career.
const PREFIX: &str = "athlete_ai_pin_v2:";
const LEGACY_PREFIX: &str = "athlete_ai_pin_v1:";
const CAREER_KEY: &str = "athlete_ai_career_v1";
const PARTICIPATION_KEY: &str = "game_sense_participation_v1";
const PARTICIPATION_DAYS_KEY: &str = "game_sense_participation_days_v1";
/// Pinned sets kept per career; older pins leave the save and the journal.
pub const PIN_SETS: usize = 64;
/// Participation not credited within this many calendar days is dropped.
pub const PARTICIPATION_DAYS: i64 = 14;
/// A champion with no binding by this tick is unbound for the rest of the set.
const SETTLE_TICKS: usize = 30;
/// Matches whose bindings stay in memory.
const RECENT_MATCHES: usize = 8;
type MatchKey = (u64, u64, u64);
#[derive(Default)]
struct Registry {
    career: String,
    /// Set by the management server; only it journals and saves pins.
    server: bool,
    latest: BTreeMap<usize, Profile>,
    pins: BTreeMap<String, Profile>,
    bindings: BTreeMap<(MatchKey, usize), Profile>,
    owners: BTreeMap<(MatchKey, usize), Profile>,
    recent: VecDeque<MatchKey>,
    dirty: BTreeMap<String, Profile>,
    development: BTreeMap<usize,crate::game_sense_core::Development>,
    athletes: BTreeMap<(MatchKey, usize),usize>,
    participation:BTreeMap<String,BTreeMap<usize,u8>>,
    /// First calendar day each participation entry was seen by management.
    participation_days: BTreeMap<String,u64>,
    saved_participation: Option<(String, String)>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}
/// The profile used when no athlete snapshot or pin exists: neutral values, current version.
pub fn neutral() -> Profile {
    Profile::default().with_sense(Default::default())
}
fn integer_field(text: &str, key: &str) -> Option<i64> {
    let token = format!("\"{key}\"");
    let tail = text
        .split_once(&token)?
        .1
        .trim_start()
        .strip_prefix(':')?
        .trim_start();
    let end = tail
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(tail.len());
    tail[..end].parse().ok()
}
fn parse_profile(text: &str) -> Profile {
    let mut p = Profile::default();
    let mut count = 0;
    for (i, key) in PROFILE_FIELDS.iter().enumerate() {
        if let Some(v) = integer_field(text, key) {
            p.values[i] = v.clamp(0, 100);
            count += 1;
        }
    }
    if count == PROFILE_FIELDS.len() {
        Profile::new(p.values)
    } else {
        Profile::default()
    }
}
fn encode(p: Profile) -> String {
    format!(
        "{} {} {}{}",
        p.version,
        p.known as u8,
        p.values
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(" "),
        if p.version>=2 {format!(" {}",p.sense.0.iter().map(i64::to_string).collect::<Vec<_>>().join(" "))} else {String::new()}
    )
}
fn decode(text: &str) -> Option<Profile> {
    let v: Vec<i64> = text
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    if !((v.len()==11&&v[0]==1)||(v.len()==15&&v[0]==2)) || v[2..].iter().any(|n|!(0..=100).contains(n)) {
        return None;
    }
    Some(Profile {
        values: v[2..11].try_into().ok()?,
        known: v[1] != 0,
        version:v[0],
        sense:if v[0]==2{crate::game_sense_core::Ratings(v[11..15].try_into().ok()?)}else{Default::default()},
    })
}
fn pin_key(m: MatchKey, athlete: usize) -> String {
    format!("{PREFIX}{}:{}:{}:{athlete}", m.0, m.1, m.2)
}
/// A v2 key for a saved or journalled pin key; v1 keys drop their career segment.
fn migrate_key(key: &str) -> Option<String> {
    if key.starts_with(PREFIX) {
        return Some(key.to_string());
    }
    let (_, rest) = key.strip_prefix(LEGACY_PREFIX)?.split_once(':')?;
    Some(format!("{PREFIX}{rest}"))
}
fn pin_set(key: &str) -> Option<MatchKey> {
    let v: Vec<u64> = key.strip_prefix(PREFIX)?.split(':').map(str::parse).collect::<Result<_, _>>().ok()?;
    (v.len() == 4).then(|| (v[0], v[1], v[2]))
}
/// Keeps the pins of the newest `PIN_SETS` sets (by match, set, seed); returns the dropped keys.
fn retain_recent(pins: &mut BTreeMap<String, Profile>) -> Vec<String> {
    let sets: BTreeSet<MatchKey> = pins.keys().filter_map(|k| pin_set(k)).collect();
    let keep: BTreeSet<MatchKey> = sets.into_iter().rev().take(PIN_SETS).collect();
    let dropped: Vec<String> = pins
        .keys()
        .filter(|k| !pin_set(k).is_some_and(|m| keep.contains(&m)))
        .cloned()
        .collect();
    for k in &dropped {
        pins.remove(k);
    }
    dropped
}
/// Reads pins from save entries and the career journal. Save entries win over the journal.
/// Returns the kept pins and the number of journal lines read.
fn load_pins(
    career: &str,
    saved: impl IntoIterator<Item = (String, String)>,
) -> (BTreeMap<String, Profile>, usize) {
    let mut pins = BTreeMap::new();
    for (k, v) in saved {
        if let (Some(k), Some(p)) = (migrate_key(&k), decode(&v)) {
            pins.insert(k, p);
        }
    }
    let mut lines = 0;
    if let Some(text) = journal(career).and_then(|path| read_journal(&path)) {
        for line in text.lines() {
            lines += 1;
            if let Some((k, v)) = line.split_once('\t') {
                if let (Some(k), Some(p)) = (migrate_key(k), decode(v)) {
                    pins.entry(k).or_insert(p);
                }
            }
        }
    }
    retain_recent(&mut pins);
    (pins, lines)
}
/// The journal's text, read from disk only when its size or modified time changed since the last read (the
/// in-game panel asks every 2 seconds; round 141).
fn read_journal(path: &std::path::Path) -> Option<String> {
    type Seen = (std::path::PathBuf, u64, Option<std::time::SystemTime>, String);
    static LAST: Mutex<Option<Seen>> = Mutex::new(None);
    let meta = std::fs::metadata(path).ok()?;
    let (len, modified) = (meta.len(), meta.modified().ok());
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((p, l, m, text)) = last.as_ref() {
        if p == path && *l == len && *m == modified && modified.is_some() {
            return Some(text.clone());
        }
    }
    let text = std::fs::read_to_string(path).ok()?;
    *last = Some((path.to_path_buf(), len, modified, text.clone()));
    Some(text)
}
/// Rewrites the journal with exactly the kept pins (temp file, then rename).
fn compact_journal(career: &str, pins: &BTreeMap<String, Profile>) {
    let Some(path) = journal(career) else { return };
    let text: String = pins.iter().map(|(k, p)| format!("{k}\t{}\n", encode(*p))).collect();
    let temp = path.with_extension("tmp");
    if std::fs::write(&temp, text).is_ok() {
        let _ = std::fs::rename(temp, path);
    }
}
/// Days since 0000-03-01 for a YYYYMMDD calendar day.
pub fn ordinal(day: u64) -> Option<i64> {
    let (y, m, d) = ((day / 10000) as i64, (day / 100 % 100) as i64, (day % 100) as i64);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    Some(era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy)
}
/// Stamps new participation entries with today and drops entries older than `PARTICIPATION_DAYS`.
fn age_participation(r: &mut Registry, today: Option<u64>) {
    let (Some(today), Some(now)) = (today, today.and_then(ordinal)) else { return };
    let keys: Vec<String> = r.participation.keys().cloned().collect();
    for k in keys {
        let seen = *r.participation_days.entry(k.clone()).or_insert(today);
        if ordinal(seen).is_none_or(|d| now - d > PARTICIPATION_DAYS) {
            r.participation.remove(&k);
            r.participation_days.remove(&k);
        }
    }
    let Registry { participation, participation_days, .. } = r;
    participation_days.retain(|k, _| participation.contains_key(k));
}
/// The participation documents, or `None` when they equal the last ones written.
fn participation_changes(r: &Registry) -> Option<(String, String)> {
    let text = (
        serde_json::to_string(&r.participation).ok()?,
        serde_json::to_string(&r.participation_days).ok()?,
    );
    (r.saved_participation.as_ref() != Some(&text)).then_some(text)
}
/// Writes participation only when it changed since the last write.
fn save_participation(ctx: &mut StableServerCtx<'_>, r: &mut Registry) {
    let Some((a, b)) = participation_changes(r) else { return };
    if ctx.save_set_string(PARTICIPATION_KEY, &a) && ctx.save_set_string(PARTICIPATION_DAYS_KEY, &b) {
        r.saved_participation = Some((a, b));
    }
}
/// A saved development entry: `Ok(None)` when absent, `Err` when present but unreadable.
fn read_development(raw: Option<&str>) -> Result<Option<crate::game_sense_core::Development>, ()> {
    raw.map_or(Ok(None), |s| crate::game_sense_core::Development::decode(s).map(Some).ok_or(()))
}
/// Game-sense overrides written by the editor (`PUT /api/game-sense`), applied once per career.
const OVERRIDE_FILE: &str = "game_sense_overrides_v1.json";
const OVERRIDE_APPLIED: &str = "game_sense_overrides_v1.applied.json";
#[derive(Debug, PartialEq)]
pub(crate) struct Override {
    pub id: usize,
    pub ratings: Option<[i64; 4]>,
    pub focus: Option<usize>,
}
/// The career and the athletes of an override file; `None` when it is not a version-1 override.
pub(crate) fn parse_overrides(text: &str) -> Option<(String, Vec<Override>)> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("version")?.as_u64()? != 1 {
        return None;
    }
    let career = match v.get("career")? {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => return None,
    };
    let mut out = Vec::new();
    for a in v.get("athletes")?.as_array()? {
        let id = a.get("id")?.as_u64()? as usize;
        let ratings = match a.get("ratings") {
            Some(serde_json::Value::Array(r)) if r.len() == 4 => {
                let mut x = [0i64; 4];
                for (i, n) in r.iter().enumerate() {
                    x[i] = n.as_f64()?.round() as i64;
                }
                Some(x)
            }
            None | Some(serde_json::Value::Null) => None,
            _ => return None,
        };
        let focus = match a.get("focus") {
            Some(f) if !f.is_null() => Some(f.as_u64()? as usize),
            _ => None,
        };
        out.push(Override { id, ratings, focus });
    }
    Some((career, out))
}
/// Ratings are clamped to 0-100 and the focus to 0-3; progress toward the next point is kept.
pub(crate) fn apply_override(d: &mut crate::game_sense_core::Development, o: &Override) {
    if let Some(r) = o.ratings {
        d.ratings = crate::game_sense_core::Ratings(r.map(|n| n.clamp(0, 100)));
    }
    if let Some(f) = o.focus {
        d.focus = f.min(3);
    }
}
/// Applies a pending editor override for this career, then renames it so it applies once.
/// A file for another career stays where it is (logged once).
fn apply_overrides(ctx: &mut StableServerCtx<'_>) {
    static SKIPPED: Mutex<Option<String>> = Mutex::new(None);
    let Some(dir) = crate::mod_dir() else { return };
    let path = dir.join(OVERRIDE_FILE);
    let Ok(text) = std::fs::read_to_string(&path) else { return };
    let Some((career, list)) = parse_overrides(&text) else {
        let _ = std::fs::rename(&path, dir.join("game_sense_overrides_v1.rejected.json"));
        log("game-sense override: unreadable file set aside as game_sense_overrides_v1.rejected.json");
        return;
    };
    let mut r = registry().lock().unwrap_or_else(|e| e.into_inner());
    if r.career.is_empty() {
        return;
    }
    if career != r.career {
        let mut s = SKIPPED.lock().unwrap_or_else(|e| e.into_inner());
        if s.as_deref() != Some(text.as_str()) {
            log(&format!("game-sense override for career {career} left in place: this career is {}", r.career));
            *s = Some(text);
        }
        return;
    }
    let mut applied = Vec::new();
    for o in &list {
        if !r.development.contains_key(&o.id) {
            let Some(stat) = ctx.athlete_get_json(o.id, "stat") else { continue };
            let profile = parse_profile(&stat);
            let seed = crate::game_sense_core::identity_seed(&r.career, o.id);
            let dev = crate::game_sense_core::Development::new(crate::game_sense_core::Ratings::initial(profile.values, seed));
            r.development.insert(o.id, dev);
        }
        let Some(d) = r.development.get_mut(&o.id) else { continue };
        apply_override(d, o);
        let encoded = d.encode();
        if ctx.save_set_string(&format!("game_sense_v1:{}", o.id), &encoded) {
            applied.push(o.id);
        }
    }
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let record = serde_json::json!({"version": 1, "career": career, "appliedUnix": stamp, "applied": applied, "requested": list.len()});
    let _ = std::fs::write(dir.join(OVERRIDE_APPLIED), serde_json::to_vec_pretty(&record).unwrap_or_default());
    let _ = std::fs::remove_file(&path);
    log(&format!("game-sense override applied to {} of {} athletes", applied.len(), list.len()));
    export_profiles(&r);
}
fn log(line: &str) {
    if let Some(dir) = crate::mod_dir() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("athlete_ai_profiles.log"))
        {
            let _ = writeln!(f, "{line}");
        }
    }
}
pub struct ProfileServer;
impl ProfileServer {
    fn refresh(ctx: &mut StableServerCtx<'_>) {
        let mut latest = BTreeMap::new();
        for id in ctx.record_ids(mod_api_stable::RecordKindV1::Athlete) {
            if let Some(text) = ctx.athlete_get_json(id, "stat") {
                let mut profile=parse_profile(&text);
                if profile.known {
                    let mut r=registry().lock().unwrap_or_else(|e|e.into_inner());
                    if !r.development.contains_key(&id) {
                        let key=format!("game_sense_v1:{id}");
                        let raw=ctx.save_get_string(&key);
                        let saved=read_development(raw.as_deref());
                        if let (Some(raw),Err(()))=(raw.as_deref(),&saved) {
                            // Keep what could not be read; the athlete restarts from derived ratings.
                            let backup=format!("game_sense_v1_unreadable:{id}");
                            if !ctx.save_contains_key(&backup) {ctx.save_set_string(&backup,raw);}
                            log(&format!("{key}: unreadable entry kept as {backup}; development restarted"));
                        }
                        let seed=crate::game_sense_core::identity_seed(&r.career,id);
                        let dev=saved.ok().flatten().unwrap_or_else(||crate::game_sense_core::Development::new(crate::game_sense_core::Ratings::initial(profile.values,seed)));
                        let encoded=dev.encode();
                        if raw.as_deref()!=Some(&encoded){ctx.save_set_string(&key,&encoded);}
                        r.development.insert(id,dev);
                    }
                    profile=profile.with_sense(r.development[&id].ratings);
                }
                latest.insert(id, profile);
            }
        }
        let today=crate::game_sense_host::today();
        let pending = if let Ok(mut r) = registry().lock() {
            r.latest = latest;
            age_participation(&mut r,today);
            save_participation(ctx,&mut r);
            export_profiles(&r);
            std::mem::take(&mut r.dirty)
        } else {
            BTreeMap::new()
        };
        for (key, p) in pending {
            if !ctx.save_set_string(&key, &encode(p)) {
                if let Ok(mut r) = registry().lock() {
                    r.dirty.insert(key, p);
                }
            }
        }
    }
}
impl StableServerExtension for ProfileServer {
    fn on_server_start(&self, ctx: &mut StableServerCtx<'_>) {
        let career = ctx
            .save_get_string(CAREER_KEY)
            .unwrap_or_else(|| {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos());
                let key = format!("{stamp:x}");
                ctx.save_set_string(CAREER_KEY, &key);
                key
            });
        let keys: Vec<String> = ctx
            .save_keys()
            .into_iter()
            .filter(|k| k.starts_with(PREFIX) || k.starts_with(LEGACY_PREFIX))
            .collect();
        let saved: Vec<(String, String)> = keys
            .iter()
            .filter_map(|k| Some((k.clone(), ctx.save_get_string(k)?)))
            .collect();
        let (pins, lines) = load_pins(&career, saved);
        // Legacy and dropped keys leave the save; kept pins missing from it are written as v2.
        for k in &keys {
            if !pins.contains_key(k) {
                ctx.save_remove_key(k);
            }
        }
        let dirty: BTreeMap<String, Profile> = pins
            .iter()
            .filter(|(k, _)| !keys.contains(k))
            .map(|(k, p)| (k.clone(), *p))
            .collect();
        if lines != pins.len() {
            compact_journal(&career, &pins);
        }
        let participation=ctx.save_get_string(PARTICIPATION_KEY).and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default();
        let participation_days=ctx.save_get_string(PARTICIPATION_DAYS_KEY).and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default();
        if let Ok(mut r) = registry().lock() {
            *r = Registry {
                career,
                server: true,
                pins,
                dirty,
                participation,
                participation_days,
                ..Registry::default()
            };
        }
        Self::refresh(ctx);
        crate::game_sense_host::reset(ctx);
        crate::game_sense_host::before(ctx);
    }
    fn before_management_tick(&self, ctx: &mut StableServerCtx<'_>) {
        apply_overrides(ctx);
        crate::game_sense_host::before(ctx)
    }
    fn after_management_tick(&self, ctx: &mut StableServerCtx<'_>) {
        let cursor=ctx.save_get_string("game_sense_event_cursor_v1").and_then(|s|s.parse().ok()).unwrap_or(0);
        let events=ctx.management_events_after(cursor);
        let finished=events.iter().any(|e|e.kind==Some(mod_api_stable::ManagementEventKindV1::MatchFinished));
        Self::refresh(ctx);
        crate::game_sense_host::after(ctx,finished);
        for e in events{
            if e.kind==Some(mod_api_stable::ManagementEventKindV1::MatchFinished){
                if let Ok(v)=serde_json::from_str::<serde_json::Value>(&e.payload_json){if let Some(id)=v.get("match_id").and_then(|v|v.as_u64()){complete_match(ctx,id);}}
            }
            ctx.save_set_string("game_sense_event_cursor_v1",&e.seq.to_string());
        }
    }
    fn handle_command(&self,ctx:&mut StableServerCtx<'_>,c:&mod_api_stable::StableCommand<'_>)->mod_api_stable::CommandResultV1{
        crate::game_sense_host::command(ctx,c)
    }
}
/// A client process does not run the management server: it reads the same pins from the save
/// and the career journal, so its match view and replays bind the profiles presim used.
pub fn load_client_pins(c: &StableClient) {
    if registry().lock().map_or(true, |r| r.server) {
        return;
    }
    let Some(career) = c.save_get_string(CAREER_KEY) else { return };
    let saved: Vec<(String, String)> = c
        .save_keys()
        .into_iter()
        .filter(|k| k.starts_with(PREFIX) || k.starts_with(LEGACY_PREFIX))
        .filter_map(|k| Some((k.clone(), c.save_get_string(&k)?)))
        .collect();
    let (pins, _) = load_pins(&career, saved);
    let Ok(mut r) = registry().lock() else { return };
    if r.server {
        return;
    }
    r.career = career;
    for (k, p) in pins {
        r.pins.entry(k).or_insert(p);
    }
}
fn match_key(sim: &StableSim<'_>) -> MatchKey {
    let o = sim.sim_origin().unwrap_or_default();
    (o.match_id, o.set_index, sim.seed())
}
fn journal(career: &str) -> Option<std::path::PathBuf> {
    if career.is_empty() || !career.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(crate::mod_dir()?.join(format!("athlete_ai_profiles_v1_{career}.txt")))
}
/// Forgets the bindings of matches older than the `RECENT_MATCHES` newest ones seen.
fn note_match(r: &mut Registry, m: MatchKey) {
    if r.recent.contains(&m) {
        return;
    }
    r.recent.push_back(m);
    while r.recent.len() > RECENT_MATCHES {
        if let Some(old) = r.recent.pop_front() {
            r.bindings.retain(|(k, _), _| *k != old);
            r.owners.retain(|(k, _), _| *k != old);
            r.athletes.retain(|(k, _), _| *k != old);
        }
    }
}
fn pin(r: &mut Registry, sim: &StableSim<'_>, key: String, athlete: usize) -> Profile {
    if let Some(p) = r.pins.get(&key) {
        return *p;
    }
    let replay = sim
        .sim_origin()
        .is_some_and(|o| o.kind == mod_api_stable::SimOriginKindV1::ClientReplay.code());
    let p = if replay {
        neutral()
    } else {
        r.latest.get(&athlete).copied().unwrap_or_else(neutral)
    };
    if r.server {
        // Persist on the first pin, before returning it to either simulation. The
        // career mod-save copy is flushed at the next management callback as well.
        if let Some(path) = journal(&r.career) {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(f, "{key}\t{}", encode(p));
                let _ = f.sync_data();
            }
        }
        if !p.known {
            log(&format!(
                "{key}: neutral fallback; {}",
                if replay {
                    "recorded profile unavailable"
                } else {
                    "supported athlete stat snapshot unavailable or incomplete"
                }
            ));
        }
        r.dirty.insert(key.clone(), p);
    }
    r.pins.insert(key, p);
    p
}
pub fn bind(sim: &StableSim<'_>, player: usize, athlete: usize) -> Profile {
    let Ok(mut r) = registry().lock() else {
        return neutral();
    };
    let m = match_key(sim);
    if let Some(p) = r.bindings.get(&(m, player)) {
        return *p;
    }
    note_match(&mut r, m);
    let profile = pin(&mut r, sim, pin_key(m, athlete), athlete);
    r.bindings.insert((m, player), profile);
    r.athletes.insert((m, player), athlete);
    profile
}
fn owner_player(sim: &StableSim<'_>, owner: usize) -> Option<usize> {
    (0..sim.player_count())
        .filter_map(|n| sim.player_at(n))
        .find(|p| p.champion().is_some_and(|c| c.id() == owner))
        .map(|p| p.id())
}
pub fn owner_athlete(sim:&StableSim<'_>,owner:usize)->Option<usize>{
 let player=owner_player(sim,owner)?;
 registry().lock().ok()?.athletes.get(&(match_key(sim),player)).copied()
}
/// Observation affects management development only, never the simulation being observed.
pub fn participate(sim:&StableSim<'_>,owner:usize,mask:u8){
 let Some(o)=sim.sim_origin()else{return};
 if !matches!(mod_api_stable::SimOriginKindV1::from_code(o.kind),Some(mod_api_stable::SimOriginKindV1::ServerPresim|mod_api_stable::SimOriginKindV1::ClientMatchView)){return;}
 if o.match_id==u64::MAX||o.set_index==u64::MAX{return;}
 let Some(athlete)=owner_athlete(sim,owner)else{return};let Ok(mut r)=registry().lock()else{return};
 let v=r.participation.entry(format!("{}:{}",o.match_id,o.set_index)).or_default().entry(athlete).or_default();*v|=mask&15;
}
pub fn complete_training(ctx:&mut StableServerCtx<'_>,id:usize,day:u64){
 if let Ok(mut r)=registry().lock(){if let Some(d)=r.development.get_mut(&id){if d.training(day){ctx.save_set_string(&format!("game_sense_v1:{id}"),&d.encode());}}}
}
pub fn set_focus(ctx:&mut StableServerCtx<'_>,id:usize,focus:usize){
 if focus>=4{return;}if let Ok(mut r)=registry().lock(){if let Some(d)=r.development.get_mut(&id){d.focus=focus;ctx.save_set_string(&format!("game_sense_v1:{id}"),&d.encode());}}
}
fn complete_match(ctx:&mut StableServerCtx<'_>,id:u64){
 use mod_api_stable::RecordKindV1 as K;
 let Some(date)=ctx.record_get_json(K::Match,id as usize,"date").and_then(|s|crate::game_sense_host::day(&s))else{return};
 let Some(sets)=ctx.record_get_json(K::Match,id as usize,"replays").and_then(|s|serde_json::from_str::<Vec<serde_json::Value>>(&s).ok()).map(|v|v.len())else{return};
 let Ok(mut r)=registry().lock()else{return};
 for n in 0..sets{
  let key=format!("{id}:{n}");r.participation_days.remove(&key);let Some(observed)=r.participation.remove(&key)else{continue};
  for(athlete,mask)in observed{if let Some(d)=r.development.get_mut(&athlete){if d.learn(date,&key,mask){ctx.save_set_string(&format!("game_sense_v1:{athlete}"),&d.encode());}}}
 }
 save_participation(ctx,&mut r);
}
fn export_profiles(r:&Registry){
 let Some(dir)=crate::mod_dir()else{return};
 let athletes:Vec<_>=r.development.iter().map(|(id,d)|serde_json::json!({"id":id,"ratings":d.ratings.0,"progress":d.progress,"focus":d.focus,"existing":r.latest.get(id).map(|p|p.values),"version":2})).collect();
 let value=serde_json::json!({"version":2,"career":r.career,"athletes":athletes,"source":"management snapshot; editor changes do not mutate live careers"});
 // round 141: compared with the bytes last written, in memory (it read the file back every management tick);
 // the file on disk is still checked once per process, so an edited or deleted export is rewritten
 static WRITTEN:Mutex<Option<Vec<u8>>>=Mutex::new(None);
 if let Ok(bytes)=serde_json::to_vec(&value){
  let mut written=WRITTEN.lock().unwrap_or_else(|e|e.into_inner());
  if written.as_deref()==Some(bytes.as_slice()){return;}
  let path=dir.join("game_sense_export_v2.json");
  if std::fs::read(&path).ok().as_deref()!=Some(bytes.as_slice()){let temp=dir.join("game_sense_export_v2.tmp");if std::fs::write(&temp,&bytes).is_err()||std::fs::rename(temp,&path).is_err(){return;}}
  *written=Some(bytes);
 }
}
/// The owner's pinned profile and whether it is final for this set. A champion with no
/// binding is neutral; that becomes final once the set is `SETTLE_TICKS` old.
fn owner_profile(sim: &StableSim<'_>, owner: usize) -> (Profile, bool) {
    let m = match_key(sim);
    let Ok(mut r) = registry().lock() else {
        return (neutral(), false);
    };
    if let Some(p) = r.owners.get(&(m, owner)) {
        return (*p, true);
    }
    let player = owner_player(sim, owner);
    let bound = player.and_then(|id| r.bindings.get(&(m, id)).copied());
    let settled = bound.is_some() || player.is_none() || sim.tick() >= SETTLE_TICKS;
    let p = bound.unwrap_or_else(neutral);
    if settled {
        note_match(&mut r, m);
        r.owners.insert((m, owner), p);
    }
    (p, settled)
}
pub fn pinned_owner(sim: &StableSim<'_>, owner: usize, cache: &mut Option<Profile>) -> Profile {
    if let Some(p) = cache {
        return *p;
    }
    let (p, settled) = owner_profile(sim, owner);
    if settled {
        *cache = Some(p);
    }
    p
}
// Wired into every kit by the shared champion model; unused until then.
#[allow(dead_code)]
/// The nine athlete values and four game-sense ratings that steer a champion's decisions,
/// never its combat numbers. Pinned per set, so presim, match view and replay agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Traits {
    pub ego: i64,
    pub judgement: i64,
    pub positioning: i64,
    pub control_speed: i64,
    pub skill_hit: i64,
    pub skill_avoid: i64,
    pub aggressive: i64,
    pub order: i64,
    pub roaming: i64,
    /// Macro, Micro, Map Awareness, Strategy.
    pub sense: [i64; 4],
    /// False for the neutral fallback (no athlete snapshot or pin).
    pub known: bool,
}
impl From<Profile> for Traits {
    fn from(p: Profile) -> Self {
        let v = p.values;
        Self {
            ego: v[0],
            judgement: v[1],
            positioning: v[2],
            control_speed: v[3],
            skill_hit: v[4],
            skill_avoid: v[5],
            aggressive: v[6],
            order: v[7],
            roaming: v[8],
            sense: p.sense.0,
            known: p.known,
        }
    }
}
impl Default for Traits {
    fn default() -> Self {
        neutral().into()
    }
}
/// The champion's traits, cached per set (see `owner_profile`).
#[allow(dead_code)]
pub fn traits(sim: &StableSim<'_>, me: usize) -> Traits {
    owner_profile(sim, me).0.into()
}

fn scene_visible_caster(sim: &StableSim<'_>, team: usize, id: usize) -> bool {
    sim.get_entity(id)
        .is_some_and(|e| e.team() == team || sim.is_visible(team, id))
}
#[derive(Clone, Default)]
pub struct Tracker {
    previous: BTreeMap<usize, (usize, i64, i64, i64)>,
    trends: BTreeMap<usize, (usize, i64)>,
    pub scene: Scene,
}
impl Tracker {
    pub fn observe(&mut self, sim: &StableSim<'_>, owner: usize) -> Scene {
        let tick = sim.tick();
        let team = sim.get_entity(owner).map_or(usize::MAX, |e| e.team());
        let mut scene = Scene::default();
        let mut previous = BTreeMap::new();
        for i in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(i) else { continue };
            if !e.is_alive() {
                continue;
            }
            let visible = e.team() == team || sim.is_visible(team, e.id());
            if !visible {
                continue;
            }
            let pos = e.pos();
            let point = (pos.0 as i64, pos.1 as i64);
            let hp = e.hp();
            let old = self.previous.get(&e.id()).copied();
            let (dt, vx, vy, loss) = old.map_or((0, 0, 0, 0), |(t, x, y, h)| {
                let dt = tick.saturating_sub(t);
                if dt == 0 || dt > 60 {
                    (dt, 0, 0, 0)
                } else {
                    (
                        dt,
                        (point.0 - x) / dt as i64,
                        (point.1 - y) / dt as i64,
                        ((h - hp.0 as i64).max(0) * 60 / dt as i64).min(hp.1 as i64 * 4),
                    )
                }
            });
            let _ = dt;
            if e.is_champion() {
                let stat = e.stat();
                let controlled = (0..e.cc_count()).filter_map(|n| e.cc_at(n)).any(|c| {
                    [
                        mod_api_stable::CcKindV1::Stun,
                        mod_api_stable::CcKindV1::Airborne,
                        mod_api_stable::CcKindV1::Bind,
                        mod_api_stable::CcKindV1::Fear,
                        mod_api_stable::CcKindV1::Charm,
                    ]
                    .iter()
                    .any(|k| c.kind == k.code())
                });
                let buffs: Vec<_> = (0..e.buff_count()).filter_map(|n| e.buff_at(n)).collect();
                scene.actors.push(Actor {
                    id: e.id(),
                    team: e.team(),
                    point,
                    hp: hp.0 as i64,
                    max_hp: hp.1 as i64,
                    attack: stat.attack as i64,
                    speed: stat.move_speed.max(1) as i64,
                    vx: if vx.abs() < 6000 { vx } else { 0 },
                    vy: if vy.abs() < 6000 { vy } else { 0 },
                    visible,
                    controlled,
                    pressure: loss / 2,
                    immune: !e.is_targetable()
                        || buffs.iter().any(|b| {
                            b.damaged_reduce >= 100
                                || (b.base_attack_damaged_reduce >= 100
                                    && b.skill_damaged_reduce >= 100)
                                || b.cc_immune
                        }),
                });
                previous.insert(e.id(), (tick, point.0, point.1, hp.0 as i64));
            } else if matches!(
                e.kind(),
                Some(mod_api_stable::EntityKindV1::Epic | mod_api_stable::EntityKindV1::Serpen)
            ) {
                if loss > 0 {
                    self.trends.insert(e.id(), (tick, loss));
                }
                let trend = self
                    .trends
                    .get(&e.id())
                    .filter(|(at, _)| tick.saturating_sub(*at) <= 120)
                    .map_or(0, |(_, n)| *n);
                scene.objectives.push(Objective {
                    id: e.id(),
                    kind: if e.kind() == Some(mod_api_stable::EntityKindV1::Epic) {
                        0
                    } else {
                        1
                    },
                    point,
                    hp: hp.0 as i64,
                    max_hp: hp.1 as i64,
                    loss_per_second: trend,
                });
                previous.insert(e.id(), (tick, point.0, point.1, hp.0 as i64));
            }
        }
        for u in &mut scene.actors {
            for j in 0..sim.projectile_count() {
                if let Some(p) = sim.projectile_at(j) {
                    if !p.is_end
                        && p.team != u.team
                        && scene_visible_caster(sim, team, p.caster_id)
                        && crate::athlete_ai_core::within(u.point, (p.x as i64, p.y as i64), 30000)
                    {
                        u.pressure += sim
                            .get_entity(p.caster_id)
                            .map_or(0, |e| e.stat().attack as i64);
                    }
                }
            }
        }
        scene.actors.sort_by_key(|u| u.id);
        scene.objectives.sort_by_key(|u| u.id);
        self.previous = previous;
        self.scene = scene.clone();
        scene
    }
}
pub fn publish(
    sim: &mut StableSim<'_>,
    owner: usize,
    centre: (i64, i64),
    radius: i64,
    opens: usize,
    end: usize,
    guest: Option<usize>,
) {
    if end <= sim.tick() {
        return;
    }
    let names = sim
        .get_entity(owner)
        .map(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .filter(|b| b.name().starts_with("engage_v1:"))
                .map(|b| b.name().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for n in names {
        sim.entity_remove_buff(owner, &n);
    }
    crate::steve::note_map_buff(sim.seed());
    sim.add_buff(
        owner,
        &crate::timed(
            &format!(
                "engage_v1:{}:{}:{radius}:{opens}:{end}:{}",
                centre.0,
                centre.1,
                guest.map_or(0, |g| g + 1)
            ),
            end - sim.tick() + 1,
        ),
    );
}
/// Allies wait this far outside an engage's radius.
const STAGE_RING: i64 = 16_000;
/// Omen stops following a target on foot beyond this.
const STAGE_FOLLOW_MAX: i64 = 150_000;
/// Only called for movement inputs. No casts, attacks or recalls are replaced.
pub fn staging_move(sim: &StableSim<'_>, owner: usize, dest: (i64, i64)) -> Option<(i64, i64)> {
    let me = sim.get_entity(owner)?;
    if !me.is_alive()
        || me.hp().0 * 100 < me.hp().1 * 30
        || (0..me.cc_count())
            .filter_map(|i| me.cc_at(i))
            .any(|c| c.kind == mod_api_stable::CcKindV1::Stun.code())
    {
        return None;
    }
    let at = me.pos();
    let point = (at.0 as i64, at.1 as i64);
    let t = sim.tick();
    // A movement away from a nearby visible attacker is an urgent retreat.
    if (0..sim.champion_count())
        .filter_map(|i| sim.get_entity(sim.champion_id_at(i)))
        .any(|e| {
            e.team() != me.team() && crate::realm::sees(sim, me.id(), me.team(), e.id()) && {
                let p = e.pos();
                let p = (p.0 as i64, p.1 as i64);
                crate::athlete_ai_core::within(point, p, 45000)
                    && crate::athlete_ai_core::sq_distance(dest, p)
                        > crate::athlete_ai_core::sq_distance(point, p)
            }
        })
    {
        return None;
    }
    if me.name().is_some_and(|n| n.ends_with("_omen")) {
        for b in (0..me.buff_count()).filter_map(|i| me.buff_at(i)) {
            if let Some(rest) = b.name().strip_prefix("omf:") {
                let mut p = rest.split(':');
                let id = p.next().and_then(|s| s.parse::<usize>().ok());
                let end = p.next().and_then(|s| s.parse::<usize>().ok());
                if let (Some(id), Some(end)) = (id, end) {
                    if end > t {
                        if let Some(e) = sim
                            .get_entity(id)
                            .filter(|e| e.is_alive() && sim.is_visible(me.team(), e.id()))
                        {
                            let a = e.pos();
                            let a = (a.0 as i64, a.1 as i64);
                            let range = (0..me.buff_count())
                                .filter_map(|i| me.buff_at(i))
                                .filter_map(|b| {
                                    b.name()
                                        .strip_prefix("omn_gun")
                                        .and_then(|s| s.parse::<usize>().ok())
                                })
                                .filter_map(|n| {
                                    [30000, 38000, 28000, 18000, 42000, 80000].get(n).copied()
                                })
                                .next()
                                .unwrap_or(30000);
                            // round 141: a follow target across the map is no longer followed on foot
                            if crate::athlete_ai_core::within(point, a, range + 10000)
                                || !crate::athlete_ai_core::within(point, a, STAGE_FOLLOW_MAX)
                            {
                                return None;
                            }
                            let (dx, dy) =
                                crate::unified_theory_math::unit(point.0 - a.0, point.1 - a.1);
                            return Some(crate::walls::pull_back(
                                point.0,
                                point.1,
                                a.0 + dx * range * 8 / 100000,
                                a.1 + dy * range * 8 / 100000,
                            ));
                        }
                    }
                }
            }
        }
    }
    // a player the team planner sent to a wave or a camp keeps farming
    if crate::universal_ai::assignment(sim, owner).is_some_and(|a| {
        matches!(a.0, crate::strategic_core::Goal::Wave | crate::strategic_core::Goal::Camp)
    }) {
        return None;
    }
    // round 141: teammates' engage marks come from the tick's shared champion snapshot (living champions with
    // their buffs, in champion order), not from reading every buff again for every player
    let team = me.team();
    let all = crate::champions_shared(sim);
    for c in all.iter() {
        if c.team != team || c.id == owner {
            continue;
        }
        for b in &c.buffs {
            let name = b.name();
            let Some(tail) = name.strip_prefix("engage_v1:") else {
                continue;
            };
            let v: Vec<i64> = tail.split(':').filter_map(|s| s.parse().ok()).collect();
            if v.len() != 6 || v[4] <= t as i64 || v[5] == owner as i64 + 1 {
                continue;
            }
            let centre = (v[0], v[1]);
            if !crate::athlete_ai_core::within(point, centre, 220000) {
                continue;
            }
            let distance = crate::athlete_ai_core::sq_distance(point, centre).isqrt() as i64;
            let arrival = (distance - v[2] - 12000).max(0) / me.stat().move_speed.max(1) as i64;
            if t as i64 + arrival > v[4] + 90 {
                continue;
            }
            if t + 15 >= v[4] as usize {
                return Some(crate::walls::pull_back(
                    point.0, point.1, centre.0, centre.1,
                ));
            }
            // parked at the ring: the same radius parks and releases (a 2,000 gap used to jitter between them)
            let radius = v[2] + STAGE_RING;
            if crate::athlete_ai_core::within(point, centre, radius + 3000)
                && !crate::athlete_ai_core::within(dest, centre, v[2] + 6000)
            {
                continue;
            }
            let (dx, dy) = crate::unified_theory_math::unit(point.0 - centre.0, point.1 - centre.1);
            let safe = (
                centre.0 + dx * radius / 10000,
                centre.1 + dy * radius / 10000,
            );
            if !crate::walls::wall_at(safe.0, safe.1) {
                return Some(crate::walls::pull_back(point.0, point.1, safe.0, safe.1));
            }
        }
    }
    None
}
#[cfg(test)]
mod override_tests {
    use super::*;
    use crate::game_sense_core::{Development, Ratings};

    #[test]
    fn overrides_parse_with_optional_fields_and_reject_other_versions() {
        let (career, list) = parse_overrides(r#"{"version":1,"career":"ab12","athletes":[{"id":7,"ratings":[10,200,-5,50.4]},{"id":9,"focus":2}]}"#).unwrap();
        assert_eq!(career, "ab12");
        assert_eq!(list, vec![
            Override { id: 7, ratings: Some([10, 200, -5, 50]), focus: None },
            Override { id: 9, ratings: None, focus: Some(2) },
        ]);
        assert!(parse_overrides(r#"{"version":2,"career":"x","athletes":[]}"#).is_none());
        assert!(parse_overrides(r#"{"version":1,"career":"x","athletes":[{"id":1,"ratings":[1,2]}]}"#).is_none());
        assert!(parse_overrides("not json").is_none());
        assert_eq!(parse_overrides(r#"{"version":1,"career":42,"athletes":[]}"#).unwrap().0, "42");
    }

    #[test]
    fn apply_clamps_and_keeps_what_was_not_given() {
        let mut d = Development::new(Ratings([40, 50, 60, 70]));
        d.progress = [5, 6, 7, 8];
        let focus = d.focus;
        apply_override(&mut d, &Override { id: 1, ratings: Some([120, -3, 55, 70]), focus: None });
        assert_eq!(d.ratings.0, [100, 0, 55, 70]);
        assert_eq!(d.focus, focus);
        assert_eq!(d.progress, [5, 6, 7, 8]);
        apply_override(&mut d, &Override { id: 1, ratings: None, focus: Some(9) });
        assert_eq!(d.focus, 3);
        assert_eq!(d.ratings.0, [100, 0, 55, 70]);
        assert!(Development::decode(&d.encode()).is_some(), "the result is a valid save entry");
    }

    /// The editor's descriptions (editor/athlete-play.js) use the native numbers: the reaction time of
    /// kits that cast natively, and every athlete_play threshold.
    #[test]
    fn editor_athlete_play_table_matches_the_native_numbers() {
        use crate::athlete_play::*;
        let js = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../editor/athlete-play.js")).unwrap();
        let line = |key: &str| js.lines().find(|l| l.contains(&format!("key: '{key}'"))).unwrap_or_else(|| panic!("{key} missing from the editor table")).to_owned();
        for key in crate::athlete_ai_core::PROFILE_FIELDS {
            line(key);
        }
        let slow = crate::athlete_ai_core::Profile::new([50, 50, 50, 0, 50, 50, 50, 50, 50]);
        let delay = slow.reaction_ticks().saturating_sub(15) * 2;
        assert!(line("control_speed").contains(&format!("delayAt0: {delay}")));
        assert!(js.contains("24 - Math.floor(Math.max(0, Math.min(100, v)) * 18 / 100)"));
        let has = |key: &str, field: &str, value: String| {
            let l = line(key);
            assert!(l.contains(&format!("{field}: {value}")), "{key}.{field} should be {value}: {l}");
        };
        has("skill_avoid", "below", SKILL_AVOID_BASE.to_string());
        has("positioning", "above", POSITIONING_KITE.to_string());
        has("positioning", "stepEveryTicks", KITE_EVERY.to_string());
        has("ego", "below", EGO_NEUTRAL.to_string());
        has("ego", "retreatBase", EGO_RETREAT_BASE.to_string());
        has("ego", "enterRange", EGO_ENTER_RANGE.to_string());
        has("ego", "exitRange", EGO_EXIT_RANGE.to_string());
        has("ego", "exitHpMargin", EGO_EXIT_MARGIN.to_string());
        has("aggressive", "above", AGGRESSIVE_HIGH.to_string());
        has("aggressive", "below", AGGRESSIVE_LOW.to_string());
        has("aggressive", "chaseHp", CHASE_HP.to_string());
        has("order", "from", ORDER_ONE.to_string());
        has("order", "twoFrom", ORDER_TWO.to_string());
        has("roaming", "above", ROAM_THRESHOLD.to_string());
        has("roaming", "baseReach", ROAM_REACH_BASE.to_string());
        has("roaming", "extraReach", ROAM_REACH_EXTRA.to_string());
        has("roaming", "quietRange", ROAM_START.to_string());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_profile_round_trips_and_ignores_unknown_fields() {
        let p = Profile::new([3, 99, 20, 80, 40, 50, 60, 70, 80]);
        assert_eq!(decode(&encode(p)), Some(p));
        let v2=p.with_sense(crate::game_sense_core::Ratings([12,34,56,78]));
        assert_eq!(decode(&encode(v2)),Some(v2));
        assert_eq!(decode(&encode(p)).unwrap().version,1);
        assert_eq!(decode(&encode(p)).unwrap().sense,Default::default());
        assert!(decode("2 1 50 50").is_none());
        let doc="{\"ego\":91,\"judgement\":42,\"positioning\":80,\"control_speed\":71,\"skill_hit\":66,\"skill_avoid\":55,\"aggressive\":44,\"order\":33,\"roaming\":22,\"attack\":999}";
        let q = parse_profile(doc);
        assert!(q.known);
        assert_eq!(q.values, [91, 42, 80, 71, 66, 55, 44, 33, 22]);
        assert!(!parse_profile("{}").known);
    }
    #[test]
    fn participation_save_roundtrips_both_teams_without_combining_sets(){
        let mut r=Registry::default();r.participation.insert("42:0".into(),[(101,3),(205,12)].into_iter().collect());r.participation.insert("42:1".into(),[(101,8)].into_iter().collect());
        let text=serde_json::to_string(&r.participation).unwrap();let restored:BTreeMap<String,BTreeMap<usize,u8>>=serde_json::from_str(&text).unwrap();assert_eq!(restored,r.participation);
        let mut athlete=crate::game_sense_core::Development::new(Default::default());assert!(athlete.learn(20261021,"42:0",restored["42:0"][&101]));assert!(!athlete.learn(20261022,"42:0",15));let transfer=athlete.encode();assert_eq!(crate::game_sense_core::Development::decode(&transfer),Some(athlete));
    }
    use mod_api_stable::{EntityHandleV1 as E, PlayerHandleV1 as P, SimCtxV1, SimOriginV1, SimVtableV1};
    use std::ffi::c_void;
    struct Host {
        origin: SimOriginV1,
        seed: u64,
        tick: usize,
    }
    unsafe fn host<'a>(p: *const c_void) -> &'a Host {
        &*(p as *const Host)
    }
    unsafe extern "C" fn origin(p: *const c_void, out: *mut SimOriginV1) -> bool {
        *out = host(p).origin;
        true
    }
    unsafe extern "C" fn seed(p: *const c_void) -> u64 {
        host(p).seed
    }
    unsafe extern "C" fn tick(p: *const c_void) -> usize {
        host(p).tick
    }
    unsafe extern "C" fn players(_: *const c_void) -> usize {
        2
    }
    unsafe extern "C" fn player_at(_: *const c_void, i: usize) -> P {
        P::from_id(i)
    }
    unsafe extern "C" fn player_valid(_: *const c_void, h: P) -> bool {
        h.id().is_some_and(|id| id < 2)
    }
    unsafe extern "C" fn champion(_: *const c_void, h: P) -> E {
        E::from_id(100 + h.id().unwrap())
    }
    unsafe extern "C" fn valid(_: *const c_void, h: E) -> bool {
        h.id().is_some_and(|id| (100..102).contains(&id))
    }
    /// A sim of two players (champions 100 and 101) at `tick` for match `id`, set 0, seed 77.
    fn with_sim<T>(kind: mod_api_stable::SimOriginKindV1, id: u64, tick_now: usize, f: impl FnOnce(&StableSim) -> T) -> T {
        unsafe {
            let mut o = SimOriginV1::default();
            o.kind = kind.code();
            o.match_id = id;
            o.set_index = 0;
            let h = Host { origin: o, seed: 77, tick: tick_now };
            let mut table: SimVtableV1 = std::mem::zeroed();
            table.size = std::mem::size_of::<SimVtableV1>();
            table.sim_origin = Some(origin);
            table.seed = Some(seed);
            table.tick = Some(tick);
            table.player_count = Some(players);
            table.player_at = Some(player_at);
            table.player_is_valid = Some(player_valid);
            table.player_champion = Some(champion);
            table.entity_is_valid = Some(valid);
            let mut raw = SimCtxV1 {
                size: std::mem::size_of::<SimCtxV1>(),
                sim: &table,
                frame: std::ptr::null(),
                state: &h as *const Host as *mut c_void,
            };
            f(&StableSim::from_raw(&mut raw).unwrap())
        }
    }
    use mod_api_stable::SimOriginKindV1 as Origin;
    #[test]
    fn fresh_processes_and_pinless_replays_bind_the_same_current_neutral_profile() {
        // Two fresh registries (a client process with no management server) and a replay.
        let (mut a, mut b) = (Registry::default(), Registry::default());
        let view = with_sim(Origin::ClientMatchView, 11, 0, |s| pin(&mut a, s, pin_key(match_key(s), 7), 7));
        let replay = with_sim(Origin::ClientReplay, 11, 0, |s| pin(&mut b, s, pin_key(match_key(s), 7), 7));
        assert_eq!(view, replay);
        assert_eq!(view, neutral());
        assert_eq!(view.version, crate::athlete_ai_core::PROFILE_VERSION);
        assert_eq!(Traits::from(view), Traits::default());
        // The neutral pin is cached: the athlete's later snapshot does not change this set.
        a.latest.insert(7, Profile::new([90; 9]).with_sense(crate::game_sense_core::Ratings([80; 4])));
        assert_eq!(with_sim(Origin::ClientMatchView, 11, 0, |s| pin(&mut a, s, pin_key(match_key(s), 7), 7)), view);
        // Neither registry is the server, so neither journals nor queues a save write.
        assert!(a.dirty.is_empty() && b.dirty.is_empty());
    }
    #[test]
    fn pins_are_keyed_by_the_simulation_so_any_process_finds_the_presim_pin() {
        let real = Profile::new([10, 20, 30, 40, 50, 60, 70, 80, 90]).with_sense(crate::game_sense_core::Ratings([1, 2, 3, 4]));
        // Presim on the server: the athlete's snapshot is pinned.
        let mut server = Registry { server: false, ..Registry::default() };
        server.latest.insert(7, real);
        let key = with_sim(Origin::ServerPresim, 12, 0, |s| pin_key(match_key(s), 7));
        assert_eq!(key, "athlete_ai_pin_v2:12:0:77:7");
        assert_eq!(with_sim(Origin::ServerPresim, 12, 0, |s| pin(&mut server, s, key.clone(), 7)), real);
        // A client and a replay load the save (a v1 key that carried the career) and agree.
        let legacy = vec![("athlete_ai_pin_v1:abc123:12:0:77:7".to_string(), encode(real))];
        let (pins, _) = load_pins("", legacy);
        assert_eq!(pins.get(&key), Some(&real));
        for kind in [Origin::ClientMatchView, Origin::ClientReplay] {
            let mut client = Registry { pins: pins.clone(), ..Registry::default() };
            assert_eq!(with_sim(kind, 12, 0, |s| pin(&mut client, s, key.clone(), 7)), real);
        }
    }
    #[test]
    fn unbound_owners_are_neutral_and_cached_once_the_set_settles() {
        // A unique match id keeps this off other tests' entries in the process registry.
        let id = 9_000_013;
        let (early, settled) = with_sim(Origin::ServerPresim, id, 0, |s| owner_profile(s, 100));
        assert_eq!((early, settled), (neutral(), false));
        let real = Profile::new([70; 9]);
        let bound = {
            let mut r = registry().lock().unwrap();
            let m = (id, 0, 77);
            r.bindings.insert((m, 0), real);
            r.bindings.get(&(m, 0)).copied()
        };
        assert_eq!(bound, Some(real));
        assert_eq!(with_sim(Origin::ServerPresim, id, 1, |s| owner_profile(s, 100)), (real, true));
        assert_eq!(with_sim(Origin::ServerPresim, id, 1, |s| traits(s, 100)), Traits::from(real));
        // Champion 101 never binds: neutral, and final from SETTLE_TICKS on.
        assert_eq!(with_sim(Origin::ServerPresim, id, SETTLE_TICKS - 1, |s| owner_profile(s, 101)), (neutral(), false));
        let mut cache = None;
        assert_eq!(with_sim(Origin::ServerPresim, id, SETTLE_TICKS, |s| pinned_owner(s, 101, &mut cache)), neutral());
        assert_eq!(cache, Some(neutral()));
    }
    #[test]
    fn pins_keep_only_the_newest_sets_and_the_journal_compacts_to_them() {
        let mut pins = BTreeMap::new();
        for m in 0..(PIN_SETS as u64 + 10) {
            for athlete in 0..3 {
                pins.insert(pin_key((m, 0, 5), athlete), neutral());
            }
        }
        pins.insert("athlete_ai_pin_v2:garbage".into(), neutral());
        let dropped = retain_recent(&mut pins);
        assert_eq!(pins.len(), PIN_SETS * 3);
        assert_eq!(dropped.len(), 10 * 3 + 1);
        assert!(pins.contains_key(&pin_key((PIN_SETS as u64 + 9, 0, 5), 0)));
        assert!(!pins.contains_key(&pin_key((9, 0, 5), 0)));
        assert!(pins.contains_key(&pin_key((10, 0, 5), 2)));
    }
    #[test]
    fn calendar_ordinals_count_real_days() {
        let d = |a: u64, b: u64| ordinal(b).unwrap() - ordinal(a).unwrap();
        assert_eq!(d(20261007, 20261021), 14);
        assert_eq!(d(20261231, 20270101), 1);
        assert_eq!(d(20240228, 20240301), 2);
        assert_eq!(d(20230228, 20230301), 1);
        assert_eq!(d(20000101, 20010101), 366);
        assert!(ordinal(20261321).is_none());
    }
    #[test]
    fn participation_older_than_two_weeks_is_dropped_and_writes_only_on_change() {
        let mut r = Registry::default();
        r.participation.insert("1:0".into(), [(5, 1)].into_iter().collect());
        age_participation(&mut r, None);
        assert!(r.participation_days.is_empty(), "no calendar yet: nothing stamped or dropped");
        age_participation(&mut r, Some(20261001));
        assert_eq!(r.participation_days["1:0"], 20261001);
        r.participation.insert("2:0".into(), [(6, 2)].into_iter().collect());
        age_participation(&mut r, Some(20261015));
        assert_eq!(r.participation.len(), 2, "14 days old is kept");
        age_participation(&mut r, Some(20261016));
        assert!(!r.participation.contains_key("1:0"));
        assert_eq!(r.participation_days.keys().collect::<Vec<_>>(), ["2:0"]);
        // Unchanged documents are not rewritten.
        let text = participation_changes(&r).unwrap();
        r.saved_participation = Some(text);
        assert!(participation_changes(&r).is_none());
        r.participation.get_mut("2:0").unwrap().insert(9, 4);
        assert!(participation_changes(&r).is_some());
    }
    #[test]
    fn unreadable_development_is_reported_not_treated_as_absent() {
        let d = crate::game_sense_core::Development::new(crate::game_sense_core::Ratings([40; 4]));
        assert_eq!(read_development(None), Ok(None));
        assert_eq!(read_development(Some(&d.encode())), Ok(Some(d)));
        assert_eq!(read_development(Some("1 9 garbage")), Err(()));
    }
}
