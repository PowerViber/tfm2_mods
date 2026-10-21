//! Supported management reads -> immutable match profiles. No private game memory or athlete writes.
use crate::athlete_ai_core::{Actor, Objective, Profile, Scene, PROFILE_FIELDS};
use mod_api_stable::{StableServerCtx, StableServerExtension, StableSim};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
const PREFIX: &str = "athlete_ai_pin_v1:";
#[derive(Default)]
struct Registry {
    career: String,
    latest: BTreeMap<usize, Profile>,
    pins: BTreeMap<String, Profile>,
    bindings: BTreeMap<String, Profile>,
    dirty: BTreeMap<String, Profile>,
    development: BTreeMap<usize,crate::game_sense_core::Development>,
    athletes: BTreeMap<String,usize>,
    participation:BTreeMap<String,BTreeMap<usize,u8>>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
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
pub struct ProfileServer;
impl ProfileServer {
    fn refresh(ctx: &mut StableServerCtx<'_>) {
        let mut latest = BTreeMap::new();
        for id in ctx.record_ids(mod_api_stable::RecordKindV1::Athlete) {
            if let Some(text) = ctx.athlete_get_json(id, "stat") {
                let mut profile=parse_profile(&text);
                if profile.known {
                    let key=format!("game_sense_v1:{id}");
                    let saved=ctx.save_get_string(&key).and_then(|s|crate::game_sense_core::Development::decode(&s));
                    let mut r=registry().lock().unwrap_or_else(|e|e.into_inner());
                    let seed=crate::game_sense_core::identity_seed(&r.career,id);
                    let dev=r.development.entry(id).or_insert_with(||saved.unwrap_or_else(||crate::game_sense_core::Development::new(crate::game_sense_core::Ratings::initial(profile.values,seed))));
                    profile=profile.with_sense(dev.ratings);
                    let encoded=dev.encode();if ctx.save_get_string(&key).as_deref()!=Some(&encoded){ctx.save_set_string(&key,&encoded);}
                }
                latest.insert(id, profile);
            }
        }
        let pending = if let Ok(mut r) = registry().lock() {
            r.latest = latest;
            if let Ok(text)=serde_json::to_string(&r.participation){ctx.save_set_string("game_sense_participation_v1",&text);}
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
            .save_get_string("athlete_ai_career_v1")
            .unwrap_or_else(|| {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos());
                let key = format!("{stamp:x}");
                ctx.save_set_string("athlete_ai_career_v1", &key);
                key
            });
        let mut pins: BTreeMap<String, Profile> = ctx
            .save_keys()
            .into_iter()
            .filter(|k| k.starts_with(PREFIX))
            .filter_map(|k| decode(&ctx.save_get_string(&k)?).map(|p| (k, p)))
            .collect();
        if let Some(path) = journal(&career) {
            if let Ok(text) = std::fs::read_to_string(path) {
                for line in text.lines() {
                    if let Some((k, v)) = line.split_once('\t') {
                        if let Some(p) = decode(v) {
                            pins.entry(k.to_string()).or_insert(p);
                        }
                    }
                }
            }
        }
        if let Ok(mut r) = registry().lock() {
            *r = Registry {
                career,
                pins,
                participation:ctx.save_get_string("game_sense_participation_v1").and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default(),
                ..Registry::default()
            };
        }
        Self::refresh(ctx);
        crate::game_sense_host::reset(ctx);
        crate::game_sense_host::before(ctx);
    }
    fn before_management_tick(&self, ctx: &mut StableServerCtx<'_>) {
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
fn match_key(sim: &StableSim<'_>) -> String {
    let o = sim.sim_origin().unwrap_or_default();
    format!("{}:{}:{}", o.match_id, o.set_index, sim.seed())
}
fn binding_key(sim: &StableSim<'_>, player: usize) -> String {
    format!("{}:{player}", match_key(sim))
}
fn journal(career: &str) -> Option<std::path::PathBuf> {
    if career.is_empty() || !career.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(crate::mod_dir()?.join(format!("athlete_ai_profiles_v1_{career}.txt")))
}
pub fn bind(sim: &StableSim<'_>, player: usize, athlete: usize) -> Profile {
    let Ok(mut r) = registry().lock() else {
        return Profile::default();
    };
    let key = format!("{PREFIX}{}:{}:{athlete}", r.career, match_key(sim));
    let profile = if let Some(p) = r.pins.get(&key) {
        *p
    } else {
        let replay = sim
            .sim_origin()
            .is_some_and(|o| o.kind == mod_api_stable::SimOriginKindV1::ClientReplay.code());
        let p = if replay {
            Profile::default()
        } else {
            r.latest.get(&athlete).copied().unwrap_or_else(||Profile::default().with_sense(Default::default()))
        };
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
            if let Some(dir) = crate::mod_dir() {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(dir.join("athlete_ai_profiles.log"))
                {
                    let _ = writeln!(
                        f,
                        "{key}: neutral fallback; {}",
                        if replay {
                            "recorded profile unavailable"
                        } else {
                            "supported athlete stat snapshot unavailable or incomplete"
                        }
                    );
                }
            }
        }
        r.pins.insert(key.clone(), p);
        r.dirty.insert(key, p);
        p
    };
    r.bindings.insert(binding_key(sim, player), profile);
    r.athletes.insert(binding_key(sim,player),athlete);
    profile
}
pub fn owner_athlete(sim:&StableSim<'_>,owner:usize)->Option<usize>{
 let player=(0..sim.player_count()).filter_map(|n|sim.player_at(n)).find(|p|p.champion().is_some_and(|c|c.id()==owner))?.id();
 registry().lock().ok()?.athletes.get(&binding_key(sim,player)).copied()
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
  let key=format!("{id}:{n}");let Some(observed)=r.participation.remove(&key)else{continue};
  for(athlete,mask)in observed{if let Some(d)=r.development.get_mut(&athlete){if d.learn(date,&key,mask){ctx.save_set_string(&format!("game_sense_v1:{athlete}"),&d.encode());}}}
 }
 if let Ok(s)=serde_json::to_string(&r.participation){ctx.save_set_string("game_sense_participation_v1",&s);}
}
fn export_profiles(r:&Registry){
 let Some(dir)=crate::mod_dir()else{return};
 let athletes:Vec<_>=r.development.iter().map(|(id,d)|serde_json::json!({"id":id,"ratings":d.ratings.0,"progress":d.progress,"focus":d.focus,"existing":r.latest.get(id).map(|p|p.values),"version":2})).collect();
 let value=serde_json::json!({"version":2,"career":r.career,"athletes":athletes,"source":"management snapshot; editor changes do not mutate live careers"});
 if let Ok(bytes)=serde_json::to_vec(&value){let path=dir.join("game_sense_export_v2.json");if std::fs::read(&path).ok().as_deref()!=Some(bytes.as_slice()){let temp=dir.join("game_sense_export_v2.tmp");if std::fs::write(&temp,bytes).is_ok(){let _=std::fs::rename(temp,path);}}}
}
pub fn pinned_owner(sim: &StableSim<'_>, owner: usize, cache: &mut Option<Profile>) -> Profile {
    if let Some(p) = cache {
        return *p;
    }
    let player = (0..sim.player_count())
        .filter_map(|i| sim.player_at(i))
        .find(|p| p.champion().is_some_and(|c| c.id() == owner))
        .map(|p| p.id());
    let p = player.and_then(|id| {
        registry()
            .lock()
            .ok()
            .and_then(|r| r.bindings.get(&binding_key(sim, id)).copied())
    });
    if let Some(p) = p {
        *cache = Some(p);
        p
    } else {
        Profile::default()
    }
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
            e.team() != me.team() && sim.is_visible(me.team(), e.id()) && {
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
                            if crate::athlete_ai_core::within(point, a, range + 10000) {
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
    for id in (0..sim.champion_count()).map(|i| sim.champion_id_at(i)) {
        let Some(c) = sim.get_entity(id) else {
            continue;
        };
        if !c.is_alive() || c.team() != me.team() || id == owner {
            continue;
        }
        for b in (0..c.buff_count()).filter_map(|i| c.buff_at(i)) {
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
            if crate::athlete_ai_core::within(point, centre, v[2] + 18000)
                && !crate::athlete_ai_core::within(dest, centre, v[2] + 6000)
            {
                continue;
            }
            let (dx, dy) = crate::unified_theory_math::unit(point.0 - centre.0, point.1 - centre.1);
            let radius = v[2] + 16000;
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
}
