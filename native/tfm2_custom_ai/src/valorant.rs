//! Omen (tfm2_valorant): a shadow controller, and the Valorant folder's shared passive, Buy Phase.
//!
//!   Buy Phase  credits instead of items (round 57): start 1000, kill +300, assist +150 (smoke / blind assists too), +200
//!              every 20 s alive, +50 for every kill his team gets, death +400 (the armor is lost, the gun is kept). He
//!              buys at his spawn, or anywhere out of combat, and
//!              buys the best gun he can pay for first, then armor from what is left (round 54). Guns change his basic attack (range, damage,
//!              attack speed); armor is a shield that only buying refills. The gun is shown over his head.
//!   S1  Dark Cover: a shadow dome (radius SMOKE_R, 100000 across) for 10 s, placed anywhere on the map. Two
//!       charges: the first comes back on the cooldown, the second is bought. He can re-smoke a spot before it fades.
//!   S2  Shadow Step or Paranoia, picked by the situation:
//!       Step      after a 0.6 s channel he teleports up to STEP_R, always to the safest spot that does the job
//!                 (round 54: away from visible enemies and enemy towers, toward teammates, bushes and his smokes);
//!                 except the risky play (round 55): behind a low enemy he can finish, cutting their retreat;
//!       Paranoia  a slow shadow (size of Purple, slower) that flies through walls and blinds EVERY champion it touches
//!                 for 5 s up close down to 3 s at its full range (round 54), his teammates too, so he only throws it on a line with no teammate on it.
//!   Ult From the Shadows: a 2 s channel, then he teleports anywhere. His shade waits at the destination; if it's
//!       killed during the channel the teleport is cancelled. He's untouchable while channelling. He lands in a bush
//!       near the spot when there is one (no warning), else in one of his smokes; smoking it first is the 2nd option.
//!
//! Smoke vision (the match hook, `fog`): nobody outside a smoke sees into it, nobody sees through it, and someone
//! inside sees only within SMOKE_SEE (and nothing outside) — their team's other eyes still count. Shooting or casting
//! from inside reveals you for REVEAL_T.
//! Blind: a blinded champion can't see. Projections ("shadows") of the enemies around them appear where those enemies
//! are and drift on the way they were moving; the blinded champion's attacks are forced onto the nearest shadow (a
//! taunt), and every shadow pops on its first hit. Skills can still be cast at the real enemies.
//! Enemy awareness (the input AI, `clear_smoke`): an enemy smoke in the way is checked before walking past it.
//!
//! The data file only sets markers (omn_s1, omn_s2, omn_ult) and their gates (omn_s1_ok, omn_s2_ok, omn_ult_ok, omn_more).

use super::*;

pub const OMEN_ID: &str = "tfm2_valorant_omen";

// ---- smokes
pub const SMOKE_R: i64 = 50_000;             // round 44: 100000 across, the size of a 100000 wall (was 26000, then 100000 radius)
const SMOKE_T: usize = 600;
const SMOKE_SEE: i64 = 15_000;
const SMOKE_MAX: usize = 3;
const SMOKE_REDRAW: usize = 30;          // the dome animation lasts 0.5 s: re-placed every 30 ticks
const RESMOKE_LEFT: usize = 120;         // a smoke with less than 2 s left may be re-smoked on the spot
const REVEAL_T: usize = 30;
const SIGHT_R: i64 = 110_000;
const SMOKE_PRICE: i64 = 200;
// ---- S2
const STEP_R: i64 = 50_000;
const STEP_T: usize = 36;
/// S2's cooldown (presets.js s2Cd): how long a Paranoia he threw by himself keeps the data's S2 shut.
const S2_LOCK: usize = 420;
const PARA_R: i64 = 26_000;
const PARA_SPEED: i64 = 3_200;
const PARA_RANGE: i64 = 180_000;
const PARA_ALLY_MARGIN: i64 = 8_000;     // champion radius + a little room for them to move (round 43: was 16000)
const BLIND_MAX: usize = 300;           // round 54: the closer, the longer: 5 s up close ...
const BLIND_MIN: usize = 180;           // ... down to 3 s at Paranoia's full range
const BLIND_CLOSE: i64 = 20_000;
const DECOY_R: i64 = 80_000;             // the blinded see shadows of enemies this close to them
// ---- ult
const ULT_T: usize = 120;
const ULT_BIND_EVERY: usize = 20;
const MAP: i64 = 960_000;
// ---- economy
// round 57: richer (he rarely reached a gun): ~600 a minute alive, more per kill / assist, a share of every team kill
const START_CREDITS: i64 = 1_000;
const KILL_CR: i64 = 300;
const ASSIST_CR: i64 = 150;
const ALIVE_CR: i64 = 200;
const ALIVE_EVERY: usize = 1_200;
const DEATH_CR: i64 = 400;
const TEAM_KILL_CR: i64 = 50;
/// He also buys away from his spawn when no enemy he can see is this close (out of combat).
const SAFE_BUY_R: i64 = 110_000;
/// Smoke / blind assists: an enemy he blinded or that stood in his smoke counts as his assist if it dies within this
/// long; a tiny tag of damage (TAG_AP, every TAG_EVERY) also puts him on the game's own assist list.
const ASSIST_WINDOW: usize = 300;
const TAG_EVERY: usize = 90;
const TAG_AP: usize = 3;
const MAX_CR: i64 = 9_000;
const BUY_R: i64 = 45_000;

/// Base champions: their spawned projections use their own sprite. Anything else (a mod champion) gets the shadow
/// figure, drawn over the spare base sprite "bombardier" (no base champion uses it).
pub(crate) const BASE_SPRITES: &[&str] = &["alchemist", "android", "archer", "astrologer", "bard", "barrier_magician", "berserker", "bomber",
    "boomerang_hunter", "cavalry_knight", "chef", "circus_blade", "clown", "crossbowman", "dancer", "dark_mage", "demon", "dokkaebi",
    "druid", "dual_blader", "enchanter", "executioner", "exorcist", "ferryman", "fighter", "gambler", "ghost", "gunner", "hammerer",
    "harpooner", "hitman", "hunter", "hypnotist", "ice_mage", "illusionist", "inquisitor", "jiangshi", "knight", "lancer",
    "lightning_mage", "magic_knight", "monk", "necromancer", "nightmare", "ninja", "ogre", "plague_doctor", "poison_dart_hunter",
    "pole_warrior", "priest", "prisoner", "pyromancer", "pythoness", "sand_mage", "shadowmancer", "shield_bearer", "siege_breaker",
    "soldier", "spellbreaker", "spirit_caller", "strongman", "swordman", "taoist", "vampire", "voodoo_shaman", "werewolf",
    "whip_master", "white_mage", "wind_mage"];
pub const SHADOW_SPRITE: &str = "bombardier";

/// Guns: (name, price, range +, attack_mult, attack_speed_mult). Classic is free.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Gun { #[default] Classic, Sheriff, Spectre, Judge, Vandal, Operator }

impl Gun {
    fn idx(self) -> usize { self as usize }
    fn price(self) -> i64 { [0, 800, 1600, 1850, 2900, 4700][self.idx()] }
    fn tier(self) -> usize { [0, 1, 2, 2, 3, 4][self.idx()] }
    /// Basic-attack reach with this gun (the data's 18000 + the gun's range buff).
    fn reach(self) -> i64 { [30_000, 38_000, 28_000, 18_000, 42_000, 80_000][self.idx()] }
    fn buff(self) -> BuffV1 {
        let mut b = BuffV1::named(&format!("omn_gun{}", self.idx()));
        // the data's basic attack has the shotgun's reach (18000); every other gun adds to it
        let (range, atk, aspd): (i64, i32, i32) = match self {
            Gun::Classic => (12_000, 0, 0),
            Gun::Sheriff => (20_000, 35, -30),
            Gun::Spectre => (10_000, -15, 60),
            Gun::Judge => (0, 0, -25),
            Gun::Vandal => (24_000, 30, 15),
            Gun::Operator => (62_000, 200, -75),
        };
        b.range = range.max(0) as usize;
        b.attack_mult = atk;
        b.attack_speed_mult = aspd;
        b
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Smoke { pub x: i64, pub y: i64, pub end: usize, pub team: usize, pub owner: usize }

#[derive(Clone, Debug)]
struct Para { x0: i64, y0: i64, dx: f64, dy: f64, range: i64, t0: usize, hit: Vec<usize> }

#[derive(Clone, Debug)]
struct Decoy { id: usize, of: usize, team: usize, x: f64, y: f64, vx: f64, vy: f64 }

#[derive(Clone, Copy, Debug)]
enum S2Plan { Para(f64, f64, i64), Step(i64, i64) }   // Paranoia: direction and how far it flies

#[derive(Clone, Debug)]
struct UltCh { x: i64, y: i64, t_end: usize, shade: Option<usize> }

#[derive(Clone, Default)]
pub struct Omen {
    started: bool,
    credits: i64,
    gun: Gun,
    armor: usize,                         // 0 none, 1 light, 2 heavy
    armor_amt: usize,
    smoke_bought: bool,
    spawn: Option<(i64, i64)>,
    alive_since: usize,
    paid_alive: usize,
    smokes: Vec<(i64, i64, usize)>,       // x, y, end
    para: Option<Para>,
    blinded: Vec<(usize, usize)>,         // victim, until
    decoys: Vec<Decoy>,
    decoy_until: usize,
    step: Option<(i64, i64, usize)>,
    ult: Option<UltCh>,
    vel: Vec<(usize, i64, i64, f64, f64)>,  // id, last x, last y, vx, vy (per tick, smoothed)
    s1_plan: Option<(i64, i64)>,
    s2_plan: Option<S2Plan>,
    ult_plan: Option<(i64, i64)>,
    ult_smoke: bool,                      // no bush or smoke to hide the arrival: a smoke first is the 2nd option
    last_buy: usize,
    tagged: Vec<(usize, usize, usize)>,   // enemy id, credit window end, next tag tick
    pending: Vec<usize>,                  // ticks a tagged enemy died: credited 3 ticks later unless the game did
    last_credit: usize,                   // tick of the last kill / assist the game credited
    kills_seen: usize,
}

fn vn(m: &Champ, n: &str) -> String {
    let id = if m.name.starts_with("tfm2_") { m.name.as_str() } else { OMEN_ID };
    format!("{id}_{n}")
}

fn fx(sim: &mut StableSim<'_>, name: &str, caster: usize, x: i64, y: i64, t: u64) {
    sim.play_view_effect(name, caster, &InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64), 0, 0, t);
}

/// A view effect both teams should see (round 59: Rian saw no smoke from the enemy side): played with a tower as the
/// caster (structures are seen by everyone), falling back to Omen himself if the game won't take that caster.
fn fx_seen(sim: &mut StableSim<'_>, name: &str, caster: usize, x: i64, y: i64, t: u64) {
    let at = InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64);
    let tower = (0..sim.tower_count()).map(|i| sim.tower_id_at(i)).find(|&id| sim.get_entity(id).map_or(false, |e| e.is_alive()));
    if let Some(tw) = tower { if sim.play_view_effect(name, tw, &at, 0, 0, t) { return; } }
    sim.play_view_effect(name, caster, &at, 0, 0, t);
}

fn fx_on(sim: &mut StableSim<'_>, name: &str, caster: usize, target: usize, t: u64) {
    sim.play_view_effect(name, caster, &InputTargetV1::target(target), 0, 0, t);
}

fn norm(dx: f64, dy: f64) -> (f64, f64) {
    let l = dx.hypot(dy);
    if l < 1e-9 { (1.0, 0.0) } else { (dx / l, dy / l) }
}

fn pct(c: &Champ) -> usize {
    if c.max_hp == 0 { 100 } else { c.hp * 100 / c.max_hp }
}

fn dist(ax: i64, ay: i64, bx: i64, by: i64) -> f64 {
    ((ax - bx) as f64).hypot((ay - by) as f64)
}

fn clampm(v: i64) -> i64 { v.clamp(0, MAP) }

pub fn is_omen(c: &Champ) -> bool {
    c.name == OMEN_ID || c.name.ends_with("_omen")
}

/// Every smoke that is up, from the "oms:x:y:end" buffs of any Omen.
pub fn smokes_up(all: &[Champ], tick: usize) -> Vec<Smoke> {
    let mut out = Vec::new();
    for c in all {
        for b in &c.buffs {
            if let Some(rest) = b.name().strip_prefix("oms:") {
                let v: Vec<i64> = rest.split(':').filter_map(|s| s.parse().ok()).collect();
                if v.len() >= 3 && (v[2].max(0) as usize) > tick {
                    out.push(Smoke { x: v[0], y: v[1], end: v[2].max(0) as usize, team: c.team, owner: c.id });
                }
            }
        }
    }
    out
}

fn inside(s: &Smoke, x: i64, y: i64) -> bool {
    d2(s.x, s.y, x, y) <= sq(SMOKE_R)
}

/// Does the segment p→q pass through the smoke (closest point of the segment within the radius)?
fn seg_hits_disk(px: i64, py: i64, qx: i64, qy: i64, cx: i64, cy: i64, r: i64) -> bool {
    let (dx, dy) = ((qx - px) as f64, (qy - py) as f64);
    let l2 = dx * dx + dy * dy;
    let t = if l2 < 1.0 { 0.0 } else { (((cx - px) as f64 * dx + (cy - py) as f64 * dy) / l2).clamp(0.0, 1.0) };
    let (nx, ny) = (px as f64 + dx * t, py as f64 + dy * t);
    (nx - cx as f64).hypot(ny - cy as f64) <= r as f64
}

// ------------------------------------------------------------------ the match hook: smoke vision

use std::collections::HashMap;
use std::sync::Mutex;
/// Per match (keyed by the seed): each champion's cooldowns last tick (to see a shot or cast) and until when they're
/// revealed.
static REVEAL: Mutex<Option<HashMap<u64, (usize, HashMap<usize, ((usize, usize, usize, usize), usize)>)>>> = Mutex::new(None);

/// Smoke vision, every 2 ticks while a smoke is up (also Steve's walls then: steve::wall_fog stands aside).
pub fn fog(sim: &mut StableSim<'_>, all: &[Champ], tick: usize) {
    let smokes = smokes_up(all, tick);
    let key = match_key(sim);
    // shots / casts (cooldowns jumping up) reveal for REVEAL_T
    let mut revealed: Vec<usize> = Vec::new();
    {
        let mut guard = match REVEAL.lock() { Ok(g) => g, Err(_) => return };
        let maps = guard.get_or_insert_with(HashMap::new);
        if smokes.is_empty() {
            maps.remove(&key);
            return;
        }
        let entry = maps.entry(key).or_insert_with(|| (tick, HashMap::new()));
        if tick < entry.0 || tick > entry.0 + 5 { *entry = (tick, HashMap::new()); }
        entry.0 = tick;
        for i in 0..sim.player_count() {
            let Some(p) = sim.player_at(i) else { continue };
            let Some(c) = p.champion() else { continue };
            let Some(cd) = p.cooldowns() else { continue };
            let id = c.id();
            let e = entry.1.entry(id).or_insert((cd, 0));
            let fired = cd.0 > e.0 .0 + 3 || cd.1 > e.0 .1 + 3 || cd.2 > e.0 .2 + 3 || cd.3 > e.0 .3 + 3;
            if fired { e.1 = tick + REVEAL_T; }
            e.0 = cd;
            if e.1 > tick { revealed.push(id); }
        }
    }
    // anyone who walks into an enemy smoke has checked it ("omc:x:y" for its team's AI)
    for c in all {
        for s in smokes.iter().filter(|s| s.team != c.team && d2(s.x, s.y, c.x, c.y) <= sq(12_000)) {
            let n = format!("omc:{}:{}", s.x, s.y);
            if !c.has(&n) { sim.add_buff(c.id, &timed(&n, s.end.saturating_sub(tick) + 2)); }
        }
    }
    if tick % 2 != 0 { return; }
    let walls = crate::steve::walls_up(all, tick);
    let teams: Vec<usize> = all.iter().map(|c| c.team).collect();
    // eyes: champions, minions and towers (summons and shadows don't count)
    let mut eyes: Vec<(usize, i64, i64)> = Vec::new();
    for i in 0..sim.entity_count() {
        if let Some(e) = sim.entity_at(i) {
            if e.is_alive() && teams.contains(&e.team()) && (e.is_champion() || e.is_minion() || e.is_tower()) {
                let (x, y) = e.pos();
                eyes.push((e.team(), x as i64, y as i64));
            }
        }
    }
    let blocked = |ax: i64, ay: i64, bx: i64, by: i64| {
        smokes.iter().any(|s| seg_hits_disk(ax, ay, bx, by, s.x, s.y, SMOKE_R))
            || walls.iter().any(|w| crate::steve::crosses(w, tick, ax, ay, bx, by))
    };
    for c in all.iter().filter(|c| !c.has("stv_riding") && !revealed.contains(&c.id)) {
        let in_smoke = smokes.iter().find(|s| inside(s, c.x, c.y));
        let mut seen = false;
        let mut watchers = 0;
        for &(_, x, y) in eyes.iter().filter(|w| w.0 != c.team && d2(w.1, w.2, c.x, c.y) <= sq(SIGHT_R)) {
            watchers += 1;
            let close = d2(x, y, c.x, c.y) <= sq(SMOKE_SEE);
            let eye_smoke = smokes.iter().find(|s| inside(s, x, y));
            let ok = match (in_smoke, eye_smoke) {
                // inside a smoke: only someone in the same smoke, close by
                (Some(s), Some(t)) => close && s.x == t.x && s.y == t.y,
                (Some(_), None) => false,
                // the watcher in a smoke sees nothing outside but what's right by them
                (None, Some(_)) => close,
                (None, None) => !blocked(x, y, c.x, c.y),
            };
            if ok { seen = true; break; }
        }
        if !seen && (watchers > 0 || in_smoke.is_some()) {
            sim.entity_set_invisible(c.id, 3);
        }
    }
}

/// Input AI (moves only): an enemy smoke in the way is checked before walking past it — Omen can be in any of them.
/// A champion of the other team whose move passes through (or right by) a smoke that nobody of theirs has checked,
/// with Omen out of their sight, walks into it first. Not when low, and not from far away.
pub fn clear_smoke(sim: &StableSim<'_>, all: &[Champ], me: &Champ, dest: (i64, i64), tick: usize) -> Option<(i64, i64)> {
    if pct(me) <= 40 { return None; }
    let smokes = smokes_up(all, tick);
    let mates: Vec<&Champ> = all.iter().filter(|c| c.team == me.team).collect();
    smokes.iter()
        .filter(|s| s.team != me.team && !inside(s, me.x, me.y) && !inside(s, dest.0, dest.1))
        .filter(|s| d2(s.x, s.y, me.x, me.y) <= sq(SMOKE_R + 50_000))
        .filter(|s| seg_hits_disk(me.x, me.y, dest.0, dest.1, s.x, s.y, SMOKE_R + 10_000))
        .filter(|s| !mates.iter().any(|a| a.has(&format!("omc:{}:{}", s.x, s.y))))
        .filter(|s| !sim.is_visible(me.team, s.owner))
        .min_by_key(|s| d2(s.x, s.y, me.x, me.y))
        .map(|s| walls::pull_back(me.x, me.y, s.x, s.y))
}

/// Input AI for Omen himself (moves only): lurking in his own smoke with a close-range gun while enemies come to it,
/// he holds still inside instead of walking out.
pub fn omen_hold(all: &[Champ], me: &Champ, tick: usize) -> Option<(i64, i64)> {
    if !is_omen(me) || pct(me) <= 40 || !(me.has("omn_gun3") || me.has("omn_gun2")) { return None; }
    let s = smokes_up(all, tick).into_iter().find(|s| s.owner == me.id && inside(s, me.x, me.y) && s.end > tick + 30)?;
    let near = all.iter().filter(|c| c.team != me.team && d2(c.x, c.y, s.x, s.y) <= sq(SMOKE_R + 30_000)).count();
    if near == 0 || near > 2 { return None; }
    Some((me.x, me.y))
}

// ------------------------------------------------------------------ native effect: Paranoia's hit

/// Paranoia touching an enemy: a marker; Omen's passive blinds them next tick (with the shadows).
#[derive(Clone, Debug)]
pub struct OmenBlind;
impl StableEffectType for OmenBlind {
    fn apply(&self, sim: &mut StableSim<'_>, _seed: u64, _caster: usize, input: InputTargetV1) {
        if input.kind == mod_api_stable::InputTargetKindV1::Target.code() {
            let champ = sim.get_entity(input.target_id).map_or(false, |e| e.is_champion());
            if champ { sim.add_buff(input.target_id, &timed("omn_bhit", 4)); }
        }
    }
}

// ------------------------------------------------------------------ the passive

impl Omen {
    fn player_cd(sim: &StableSim<'_>, player: usize) -> (usize, usize, usize, usize) {
        sim.get_player(player).and_then(|p| p.cooldowns()).unwrap_or((0, 1, 1, 1))
    }

    fn earn(&mut self, n: i64) {
        self.credits = (self.credits + n).clamp(0, MAX_CR);
    }

    /// Full buy, force buy or eco, Valorant-style. Guns are only upgraded (never a sidegrade); armor tops up.
    fn buy(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        self.last_buy = tick;
        let mut bought = false;
        let behind = sim.score_diff(m.team) < 0;
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        // melee-heavy enemies (close fighters by name) make the shotgun the force buy
        let close_fighters = enemies.iter().filter(|e| ["fighter", "knight", "berserker", "dual_blader", "lancer", "monk", "ninja",
            "swordman", "werewolf", "ogre", "gladiator", "executioner", "hammerer", "strongman", "pole_warrior", "steve", "vader", "dio", "v1", "david"]
            .iter().any(|n| e.name.ends_with(n))).count();
        // round 54: the gun comes first, armor from what is left (he used to top up heavy armor on every visit and
        // never reach a gun). The best upgrade he can pay for; the Operator only when rich (its damage per second is
        // below the Vandal's, it is a long-range pick).
        let _ = (behind, tick);
        let c = self.credits;
        let want = if c >= 5_500 {
            Gun::Operator
        } else if c >= 2_900 {
            Gun::Vandal
        } else if c >= 1_850 && close_fighters >= 2 {
            Gun::Judge
        } else if c >= 1_600 {
            Gun::Spectre
        } else if c >= 800 {
            Gun::Sheriff
        } else {
            self.gun
        };
        if want.tier() > self.gun.tier() && self.credits >= want.price() {
            self.credits -= want.price();
            self.gun = want;
            bought = true;
        }
        // armor: heavy when it leaves money, else light; refill only by buying
        let shield_now = sim.get_entity(m.id).map_or(0, |e| e.shield());
        let need = |lvl: usize| m.max_hp * if lvl == 2 { 25 } else { 12 } / 100;
        // round 59: armor never eats the gun money: before the Vandal he only buys armor with the Vandal's price left over
        let spare = if self.gun.tier() >= Gun::Vandal.tier() { self.credits } else { self.credits - Gun::Vandal.price() };
        let target_lvl = if spare >= 1_000 { 2 } else if spare >= 400 { 1 } else { 0 };
        if target_lvl > 0 && (self.armor < target_lvl || shield_now < need(self.armor.max(1)) / 2) {
            let lvl = target_lvl.max(self.armor.min(target_lvl));
            self.credits -= if lvl == 2 { 1_000 } else { 400 };
            sim.entity_clear_shield(m.id);
            self.armor = lvl;
            self.armor_amt = need(lvl);
            sim.entity_add_shield(m.id, self.armor_amt, 1_000_000);
            bought = true;
        }
        if !self.smoke_bought && self.credits >= SMOKE_PRICE {
            self.credits -= SMOKE_PRICE;
            self.smoke_bought = true;
            bought = true;
        }
        if bought { fx_on(sim, &vn(m, "buy"), m.id, m.id, 30); }
    }

    fn show_kit(&self, sim: &mut StableSim<'_>, m: &Champ) {
        for k in 0..6 {
            let n = format!("omn_gun{k}");
            if k == self.gun.idx() { if !m.has(&n) { sim.add_buff(m.id, &self.gun.buff()); } } else if m.has(&n) { sim.entity_remove_buff(m.id, &n); }
        }
        // the money over his head (round 55): "$" + thousands + hundreds + "00", one icon buff per digit
        let cr = self.credits.clamp(0, 9_999);
        let want = [if cr >= 1_000 { Some((cr / 1_000) as usize) } else { None }, Some(((cr / 100) % 10) as usize)];
        for (slot, tag) in ["k", "h"].iter().enumerate() {
            for d in 0..10 {
                let n = format!("omn_cr{tag}{d}");
                let on = want[slot] == Some(d);
                if on != m.has(&n) {
                    if on { sim.add_buff(m.id, &BuffV1::named(&n)); } else { sim.entity_remove_buff(m.id, &n); }
                }
            }
        }
        if !m.has("omn_cash") { sim.add_buff(m.id, &BuffV1::named("omn_cash")); }
        for k in 1..3 {
            let n = format!("omn_arm{k}");
            let on = self.armor == k && sim.get_entity(m.id).map_or(false, |e| e.shield() > 0);
            if on != m.has(&n) {
                if on { sim.add_buff(m.id, &BuffV1::named(&n)); } else { sim.entity_remove_buff(m.id, &n); }
            }
        }
        if self.smoke_bought != m.has("omn_more") {
            if self.smoke_bought { sim.add_buff(m.id, &BuffV1::named("omn_more")); } else { sim.entity_remove_buff(m.id, "omn_more"); }
        }
    }

    // ---------------------------------------------------------------- smokes

    fn place_smoke(&mut self, sim: &mut StableSim<'_>, m: &Champ, x: i64, y: i64, tick: usize) {
        let (x, y) = (clampm(x), clampm(y));
        // a re-smoke replaces the one on that spot; at most SMOKE_MAX up (the oldest goes)
        self.smokes.retain(|s| d2(s.0, s.1, x, y) > sq(SMOKE_R) && s.2 > tick);
        if self.smokes.len() >= SMOKE_MAX { self.smokes.remove(0); }
        self.smokes.push((x, y, tick + SMOKE_T));
        fx_seen(sim, &vn(m, "smoke_form"), m.id, x, y, 30);
        self.publish_smokes(sim, m, tick);
        // the smoke flies there as a shadow orb
        let d = dist(m.x, m.y, x, y);
        if d > 20_000.0 {
            let spec = ProjectileSpawnV1 {
                caster_id: m.id, team: m.team, x: m.x.max(0) as u64, y: m.y.max(0) as u64, radius: 1_000,
                speed: ((d / 20.0) as u64).max(1), move_kind: ProjectileMoveKindV1::Linear.code(),
                target_x: x as u64, target_y: y as u64, penetrate: true, casting_target: CastingTargetV1::None.code(),
                ..ProjectileSpawnV1::default()
            };
            sim.spawn_projectile(&vn(m, "smoke_orb"), &format!("{MOD_ID}:noop"), &spec);
        }
    }

    fn publish_smokes(&self, sim: &mut StableSim<'_>, m: &Champ, tick: usize) {
        for b in m.buffs.iter().filter(|b| b.name().starts_with("oms:")) {
            sim.entity_remove_buff(m.id, b.name());
        }
        for &(x, y, end) in self.smokes.iter().filter(|s| s.2 > tick) {
            crate::steve::note_map_buff(sim.seed());
            sim.add_buff(m.id, &timed(&format!("oms:{x}:{y}:{end}"), end - tick + 2));
        }
    }

    /// Where a smoke helps most right now, or None. In order:
    ///   1. his ult is ready with a destination: smoke it first (smoke a choke from spawn, then ult into it);
    ///   2. he's low and chased: on himself (they lose sight of him);
    ///   3. a teammate low and chased: on them;
    ///   4. a big objective with champions at it: the enemies' way in when his team is taking it, the pit itself
    ///      when the enemies are;
    ///   5. the ambush: with the shotgun (or his step ready) and enemies close, behind them (he steps in, shotgun out);
    ///   6. a fight: on the enemy hitting hardest (from inside a smoke they can't see out).
    /// Never on top of one of his smokes that still has 2+ s left.
    fn smoke_plan(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ], tick: usize, s2_ready: bool) -> Option<(i64, i64)> {
        let smokes = smokes_up(all, tick);
        let free = |x: i64, y: i64| !smokes.iter().any(|s| s.owner == m.id && d2(s.x, s.y, x, y) <= sq(SMOKE_R) && s.end > tick + RESMOKE_LEFT)
            && !walls::wall_at(x, y);
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let mates: Vec<&Champ> = all.iter().filter(|c| c.team == m.team && c.id != m.id).collect();
        let foes_at = |x: i64, y: i64, r: i64| enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(r)).count();
        // 1
        if let (Some(p), true) = (self.ult_plan, self.ult_smoke) {
            // round 67: an ult into a gank lands 18000 behind the target; the smoke goes behind them (its near edge
            // past the target, his landing spot inside), never over the target
            let p = match gank_target(sim, all, m) {
                Some((e, _)) if d2(e.x, e.y, p.0, p.1) <= sq(SMOKE_R + 5_000) => {
                    let b = enemy_base(sim, m);
                    let (dx, dy) = norm((b.0 - e.x) as f64, (b.1 - e.y) as f64);
                    let off = (SMOKE_R + 10_000) as f64;
                    walls::pull_back(e.x, e.y, clampm(e.x + (dx * off) as i64), clampm(e.y + (dy * off) as i64))
                }
                _ => p,
            };
            if free(p.0, p.1) { return Some(p); }
        }
        // 2
        if pct(m) <= 35 && foes_at(m.x, m.y, 40_000) > 0 && free(m.x, m.y) { return Some((m.x, m.y)); }
        // 3
        if let Some(a) = mates.iter().filter(|a| pct(a) <= 30 && foes_at(a.x, a.y, 35_000) > 0 && d2(a.x, a.y, m.x, m.y) <= sq(250_000))
            .min_by_key(|a| (pct(a), a.id)) {
            if free(a.x, a.y) { return Some((a.x, a.y)); }
        }
        // Everything else is weighed (round 45): the plans from the editor's Map tab against his own reads. A plan's
        // priority sets its weight (5 beats all his own reads, 1 loses to most), and a plan is only used with his team
        // there (someone of his team near the spot).
        use crate::tactics::{marks_for, team_there, Kind};
        let mut cands: Vec<(i64, (i64, i64))> = Vec::new();
        let plan_w = |prio: i64| 20 + 15 * prio;          // 35 .. 95
        for k in marks_for(sim, all, m, Kind::Smoke) {
            // a fake smoke (to deceive) needs nobody there and weighs a little less
            let fake = k.who == "fake";
            if d2(k.a.0, k.a.1, m.x, m.y) <= sq(400_000) && (fake || team_there(all, &k, m.team)) { cands.push((plan_w(k.prio) - if fake { 10 } else { 0 }, k.a)); }
        }
        // team goals: block a choke = smoke its middle; control an area = smoke the enemies' way in; ambush = smoke the
        // spot when enemies are around it and he can get there
        for k in marks_for(sim, all, m, Kind::Block) {
            let c = crate::tactics::centre(&k);
            if d2(c.0, c.1, m.x, m.y) <= sq(400_000) && team_there(all, &k, m.team) { cands.push((plan_w(k.prio), c)); }
        }
        for k in marks_for(sim, all, m, Kind::Control) {
            if d2(k.a.0, k.a.1, m.x, m.y) > sq(400_000) || !team_there(all, &k, m.team) { continue; }
            let foes: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, k.a.0, k.a.1) <= sq(k.r + 200_000)).collect();
            let from = if foes.is_empty() { enemy_base(sim, m) } else { let n = foes.len() as i64; (foes.iter().map(|e| e.x).sum::<i64>() / n, foes.iter().map(|e| e.y).sum::<i64>() / n) };
            let (dx, dy) = norm((from.0 - k.a.0) as f64, (from.1 - k.a.1) as f64);
            let off = (k.r + SMOKE_R * 6 / 10) as f64;
            cands.push((plan_w(k.prio) - 5, walls::pull_back(k.a.0, k.a.1, clampm(k.a.0 + (dx * off) as i64), clampm(k.a.1 + (dy * off) as i64))));
        }
        // cover = smoke the area over (the team works hidden); fake = a smoke to deceive, nobody needs to be there
        for k in marks_for(sim, all, m, Kind::Cover) {
            if d2(k.a.0, k.a.1, m.x, m.y) <= sq(400_000) && team_there(all, &k, m.team) { cands.push((plan_w(k.prio), k.a)); }
        }
        for k in marks_for(sim, all, m, Kind::Fake) {
            if d2(k.a.0, k.a.1, m.x, m.y) <= sq(400_000) { cands.push((plan_w(k.prio) - 10, k.a)); }
        }
        for k in marks_for(sim, all, m, Kind::Ambush) {
            if d2(k.a.0, k.a.1, m.x, m.y) <= sq(STEP_R + SMOKE_R + k.r) && foes_at(k.a.0, k.a.1, k.r + 60_000) > 0 && !walls::bush_at(k.a.0, k.a.1) {
                cands.push((plan_w(k.prio) - 10, k.a));
            }
        }
        // 4
        let obj = objective(sim, all, m);
        if let Some((ox, oy, ours)) = obj {
            if d2(ox, oy, m.x, m.y) <= sq(300_000) {
                let p = if ours {
                    // between the pit and the enemies (or toward the enemy side of the map)
                    let foes: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, ox, oy) <= sq(160_000)).collect();
                    let (tx, ty) = if foes.is_empty() {
                        let b = enemy_base(sim, m);
                        (b.0, b.1)
                    } else {
                        let n = foes.len() as i64;
                        (foes.iter().map(|e| e.x).sum::<i64>() / n, foes.iter().map(|e| e.y).sum::<i64>() / n)
                    };
                    let (dx, dy) = norm((tx - ox) as f64, (ty - oy) as f64);
                    walls::pull_back(ox, oy, ox + (dx * SMOKE_R as f64 * 0.9) as i64, oy + (dy * SMOKE_R as f64 * 0.9) as i64)
                } else {
                    // round 66 (Rian): never the whole pit while enemies are on it (his team couldn't see who was
                    // inside): half of it, the far half from his team
                    half_pit(m, &mates, ox, oy)
                };
                cands.push((50, p));
            }
        }
        // round 66: the same for any other smoke (a Map tab plan on the pit, a fight smoke) that would cover the pit
        // while visible enemies stand on it: it moves to the far half, so the near half stays in sight to pick them off
        if let Some((ox, oy, _)) = obj {
            if foes_at(ox, oy, 40_000) > 0 {
                let half = half_pit(m, &mates, ox, oy);
                for c in cands.iter_mut() {
                    if d2(c.1 .0, c.1 .1, ox, oy) < sq(SMOKE_R * 85 / 100) { c.1 = half; }
                }
            }
        }
        // 5
        let shotgun = m.has("omn_gun3") || m.has("omn_gun2");
        if (shotgun || s2_ready) && pct(m) >= 50 {
            if let Some(e) = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(STEP_R + 20_000))
                .filter(|e| foes_at(e.x, e.y, 35_000) <= 2).min_by_key(|e| (pct(e), e.id)) {
                let (dx, dy) = norm((e.x - m.x) as f64, (e.y - m.y) as f64);
                // the enemy well inside the far side of the dome: he steps in at its near edge
                let p = walls::pull_back(e.x, e.y, e.x + (dx * SMOKE_R as f64 * 0.6) as i64, e.y + (dy * SMOKE_R as f64 * 0.6) as i64);
                if dist(p.0, p.1, m.x, m.y) <= (STEP_R + SMOKE_R) as f64 { cands.push((45, p)); }
            }
        }
        // 6: a fight: over the enemy hitting hardest, pushed back off his own teammates (none of them inside)
        let fighting = mates.iter().any(|a| foes_at(a.x, a.y, 50_000) > 0) || foes_at(m.x, m.y, 60_000) > 0;
        if fighting {
            if let Some(e) = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(200_000))
                .max_by_key(|e| (e.attack + e.hp / 20, e.id)) {
                let near: Vec<&&Champ> = mates.iter().filter(|a| d2(a.x, a.y, e.x, e.y) <= sq(150_000)).collect();
                let from = if near.is_empty() { (m.x, m.y) } else { let n = near.len() as i64; (near.iter().map(|a| a.x).sum::<i64>() / n, near.iter().map(|a| a.y).sum::<i64>() / n) };
                let (dx, dy) = norm((e.x - from.0) as f64, (e.y - from.1) as f64);
                let p = walls::pull_back(e.x, e.y, clampm(e.x + (dx * SMOKE_R as f64 * 0.55) as i64), clampm(e.y + (dy * SMOKE_R as f64 * 0.55) as i64));
                if !mates.iter().any(|a| d2(a.x, a.y, p.0, p.1) <= sq(SMOKE_R + 5_000)) { cands.push((40, p)); }
            }
        }
        // round 67 (Rian): in a gank, never smoke the enemy being ganked (his team would lose sight of the target):
        // behind them (their way home and their help, the target left just outside), or over his teammates
        if let Some((e, _)) = gank_target(sim, all, m) {
            let b = enemy_base(sim, m);
            let (dx, dy) = norm((b.0 - e.x) as f64, (b.1 - e.y) as f64);
            let off = (SMOKE_R + 10_000) as f64;
            let behind = walls::pull_back(e.x, e.y, clampm(e.x + (dx * off) as i64), clampm(e.y + (dy * off) as i64));
            let near: Vec<&&Champ> = mates.iter().filter(|a| d2(a.x, a.y, e.x, e.y) <= sq(120_000)).collect();
            let spot = if !near.iter().any(|a| d2(a.x, a.y, behind.0, behind.1) <= sq(SMOKE_R)) && d2(behind.0, behind.1, e.x, e.y) > sq(SMOKE_R) {
                Some(behind)
            } else if !near.is_empty() {
                let n = near.len() as i64;
                let c = (near.iter().map(|a| a.x).sum::<i64>() / n, near.iter().map(|a| a.y).sum::<i64>() / n);
                (d2(c.0, c.1, e.x, e.y) > sq(SMOKE_R + 5_000)).then_some(c)
            } else {
                None
            };
            cands.retain(|c| d2(c.1 .0, c.1 .1, e.x, e.y) > sq(SMOKE_R + 5_000));
            if let Some(p) = spot { cands.push((48, p)); }
        }
        // macro with Steve (round 51): no smoke where the vision is already taken care of
        let covered = self.vision_covered(sim, m, all, tick);
        cands.into_iter().filter(|c| free(c.1 .0, c.1 .1) && !covered(c.1)).max_by_key(|c| c.0).map(|c| c.1)
    }

    /// Is a smoke at a spot pointless because a wall already does the job? (round 51, Rian: "if the wall blocked off
    /// the jungle, no need to smoke the choke it blocks")
    ///   a standing boat wall runs through or right by the smoke (within its radius): that choke is blocked already;
    ///   every enemy who could look at the spot (the visible ones within 250000, else their base) is behind a standing
    ///     wall: they can't see it anyway;
    ///   a teammate Steve has his ult ready and one of the team's wall plans for this very moment runs through the spot:
    ///     he'll wall it, so the smoke is kept for something else.
    fn vision_covered<'a>(&self, sim: &'a StableSim<'_>, m: &'a Champ, all: &'a [Champ], tick: usize) -> impl Fn((i64, i64)) -> bool + 'a {
        let walls = crate::steve::walls_up(all, tick);
        let seg_d = |p: (i64, i64), a: (i64, i64), b: (i64, i64)| {
            let (dx, dy) = ((b.0 - a.0) as f64, (b.1 - a.1) as f64);
            let l2 = dx * dx + dy * dy;
            let t = if l2 < 1.0 { 0.0 } else { (((p.0 - a.0) as f64 * dx + (p.1 - a.1) as f64 * dy) / l2).clamp(0.0, 1.0) };
            ((a.0 as f64 + dx * t - p.0 as f64).hypot(a.1 as f64 + dy * t - p.1 as f64)) as i64
        };
        // Steve's walls about to go up: his team's active wall plans, while his ult is ready
        let mut planned: Vec<((i64, i64), (i64, i64))> = Vec::new();
        for st in all.iter().filter(|c| c.team == m.team && c.id != m.id && c.name.ends_with("_steve") && !c.has("stv_riding")) {
            let ready = (0..sim.player_count()).filter_map(|i| sim.player_at(i))
                .find(|p| p.champion().map_or(false, |c| c.id() == st.id)).and_then(|p| p.cooldowns()).map_or(false, |c| c.3 == 0);
            if !ready { continue; }
            let mut lines = crate::tactics::marks_for(sim, all, st, crate::tactics::Kind::Wall);
            lines.extend(crate::tactics::marks_for(sim, all, st, crate::tactics::Kind::Block));
            planned.extend(lines.iter().filter(|k| crate::tactics::team_there(all, k, m.team)).map(|k| (k.a, k.b)));
        }
        let enemies: Vec<(i64, i64)> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).map(|c| (c.x, c.y)).collect();
        let base = enemy_base(sim, m);
        move |p: (i64, i64)| {
            if walls.iter().any(|w| seg_d(p, (w.ax, w.ay), (w.bx, w.by)) <= SMOKE_R) { return true; }
            if planned.iter().any(|&(a, b)| seg_d(p, a, b) <= SMOKE_R) { return true; }
            if walls.is_empty() { return false; }
            let lookers: Vec<(i64, i64)> = {
                let near: Vec<(i64, i64)> = enemies.iter().copied().filter(|e| d2(e.0, e.1, p.0, p.1) <= sq(250_000)).collect();
                if near.is_empty() { vec![base] } else { near }
            };
            lookers.iter().all(|e| walls.iter().any(|w| crate::steve::crosses(w, tick, e.0, e.1, p.0, p.1)))
        }
    }

    // ---------------------------------------------------------------- S2

    /// Enemies Paranoia would hit along (dx, dy) and how far it should fly (just past the farthest of them), or None
    /// when a teammate (not him) is anywhere on that stretch. `trade` (round 55: his side is running from a chase, so a
    /// blinded teammate loses little): teammates may be on it when it blinds at least 2 more enemies than teammates.
    fn para_line(&self, m: &Champ, all: &[Champ], visible: &[&Champ], d: (f64, f64), trade: bool) -> Option<(Vec<usize>, i64, usize)> {
        let place = |c: &Champ| {
            let (ex, ey) = ((c.x - m.x) as f64, (c.y - m.y) as f64);
            (ex * d.0 + ey * d.1, (ex * d.1 - ey * d.0).abs())
        };
        let hits: Vec<(usize, f64)> = visible.iter().filter_map(|e| { let (along, off) = place(e);
            (along > -(PARA_R as f64) && along <= PARA_RANGE as f64 && off <= (PARA_R + 8_000) as f64).then_some((e.id, along)) }).collect();
        if hits.is_empty() { return Some((Vec::new(), 0, 0)); }
        let far = hits.iter().map(|h| h.1).fold(0.0, f64::max);
        let len = ((far + 35_000.0) as i64).clamp(60_000, PARA_RANGE);
        let allies = all.iter().filter(|c| c.team == m.team && c.id != m.id).filter(|c| { let (along, off) = place(c);
            along > -((PARA_R + PARA_ALLY_MARGIN) as f64) && along <= (len + PARA_R + PARA_ALLY_MARGIN) as f64 && off <= (PARA_R + PARA_ALLY_MARGIN) as f64 }).count();
        if allies > 0 && !(trade && hits.len() >= allies + 2) { return None; }
        Some((hits.into_iter().map(|h| h.0).collect(), len, allies))
    }

    fn best_para(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ], trade: bool) -> Option<((f64, f64), Vec<usize>, i64)> {
        let visible: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)
            && !c.has("omn_blinded") && d2(c.x, c.y, m.x, m.y) <= sq(PARA_RANGE)).collect();
        let mut best: Option<((f64, f64), Vec<usize>, i64, i64)> = None;
        for e in &visible {
            let base = norm((e.x - m.x) as f64, (e.y - m.y) as f64);
            for a in [0.0f64, 0.12, -0.12, 0.25, -0.25, 0.4, -0.4] {
                let d = (base.0 * a.cos() - base.1 * a.sin(), base.0 * a.sin() + base.1 * a.cos());
                if let Some((hits, len, allies)) = self.para_line(m, all, &visible, d, trade) {
                    if hits.is_empty() { continue; }
                    // more hits first (a blinded teammate costs one and a half), then the straighter line
                    let score = hits.len() as i64 * 100 - allies as i64 * 150 - (a.abs() * 100.0) as i64;
                    if best.as_ref().map_or(true, |b| score > b.3) { best = Some((d, hits, len, score)); }
                }
            }
        }
        best.map(|(d, h, l, _)| (d, h, l))
    }

    /// Paranoia or the step, picked by the situation (round 43: both used much more often):
    ///   low and chased → Paranoia on the chasers if it can, else the step away (into one of his smokes if one is on
    ///     the way);
    ///   one of his smokes near enemies, healthy → step into it (the ambush);
    ///   Paranoia on any clean line that hits an enemy within 140000;
    ///   healthy, with a close-range gun, an enemy within reach with at most one other near it → step on them;
    ///   a low enemy within reach → step behind them;
    ///   one of his smokes within reach with enemies around it and him outside → step in to lurk.
    /// Round 60: getting around (the jungle, a rotation) with no enemy he can see near and on the move → the step along
    /// his way, to get there faster.
    fn travel_step(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ]) -> Option<(i64, i64)> {
        if pct(m) < 30 || all.iter().any(|e| e.team != m.team && sim.is_visible(m.team, e.id) && d2(e.x, e.y, m.x, m.y) <= sq(150_000)) { return None; }
        let (vx, vy) = self.velocity(m.id);
        let sp = vx.hypot(vy);
        if sp < 200.0 { return None; }
        let to = walls::pull_back(m.x, m.y, clampm(m.x + (vx / sp * STEP_R as f64) as i64), clampm(m.y + (vy / sp * STEP_R as f64) as i64));
        (d2(to.0, to.1, m.x, m.y) >= sq(STEP_R * 7 / 10)).then_some(to)
    }

    /// Paranoia on the chasers when his side is running (chased and outnumbered around him, or at half HP or less with
    /// someone on him): even through a teammate on the line (they're running, not shooting).
    fn retreat_para(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ]) -> Option<((f64, f64), i64)> {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let foes = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(60_000)).count();
        let mates_near = all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, m.x, m.y) <= sq(60_000)).count();
        let on_him = enemies.iter().any(|e| d2(e.x, e.y, m.x, m.y) <= sq(45_000));
        if !(on_him && (foes > mates_near || pct(m) <= 50)) { return None; }
        let (d, hits, len) = self.best_para(sim, m, all, true)?;
        hits.iter().any(|h| enemies.iter().any(|e| e.id == *h && d2(e.x, e.y, m.x, m.y) <= sq(70_000))).then_some((d, len))
    }

    fn s2_plan(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) -> Option<S2Plan> {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let foes_at = |x: i64, y: i64, r: i64| enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(r)).count();
        let para = self.best_para(sim, m, all, false);
        // the retreat (round 55): chased and outnumbered around him (e.g. him + 1 teammate running from 3), or low with
        // someone on him → Paranoia on the chasers even through a teammate on the line: they're running, not shooting
        if let Some((d, len)) = self.retreat_para(sim, m, all) { return Some(S2Plan::Para(d.0, d.1, len)); }
        let smokes: Vec<Smoke> = smokes_up(all, tick).into_iter().filter(|s| s.owner == m.id && s.end > tick + 60).collect();
        // the nearest point of a smoke he can step to, well inside it
        let into = |s: &Smoke, toward: (i64, i64)| -> Option<(i64, i64)> {
            let (dx, dy) = norm((toward.0 - s.x) as f64, (toward.1 - s.y) as f64);
            let c = walls::pull_back(s.x, s.y, s.x + (dx * (SMOKE_R - 25_000) as f64) as i64, s.y + (dy * (SMOKE_R - 25_000) as f64) as i64);
            let c = if d2(c.0, c.1, m.x, m.y) <= sq(STEP_R) { c } else {
                let (ux, uy) = norm((c.0 - m.x) as f64, (c.1 - m.y) as f64);
                walls::pull_back(m.x, m.y, m.x + (ux * STEP_R as f64) as i64, m.y + (uy * STEP_R as f64) as i64)
            };
            (inside(s, c.0, c.1) && d2(c.0, c.1, m.x, m.y) >= sq(15_000)).then_some(c)
        };
        // every step lands on the safest spot that does the job (round 54): candidates around the wanted point,
        // scored by visible enemies close to it, enemy towers, teammates, bushes and his smokes
        let score = |p: (i64, i64), ignore: Option<usize>| spot_score(sim, all, m, &smokes, p, ignore);
        let reach = |p: &(i64, i64)| d2(p.0, p.1, m.x, m.y) <= sq(STEP_R + 2_000) && d2(p.0, p.1, m.x, m.y) >= sq(15_000) && !walls::wall_at(p.0, p.1);
        let chasers: Vec<usize> = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(35_000)).map(|e| e.id).collect();
        if pct(m) <= 35 && !chasers.is_empty() {
            if let Some((d, hits, len)) = &para { if hits.iter().any(|h| chasers.contains(h)) { return Some(S2Plan::Para(d.0, d.1, *len)); } }
            let (cx, cy) = { let n = chasers.len() as i64; let cs: Vec<&&Champ> = enemies.iter().filter(|e| chasers.contains(&e.id)).collect();
                (cs.iter().map(|e| e.x).sum::<i64>() / n, cs.iter().map(|e| e.y).sum::<i64>() / n) };
            let home = self.spawn.unwrap_or((m.x, m.y));
            let now = dist(m.x, m.y, cx, cy);
            let mut cands: Vec<(i64, i64)> = ring(m, (m.x, m.y), STEP_R, 16);
            cands.extend(ring(m, (m.x, m.y), STEP_R * 7 / 10, 16));
            cands.extend(smokes.iter().filter(|s| !inside(s, m.x, m.y)).filter_map(|s| into(s, (m.x, m.y))));
            let best = cands.into_iter().filter(|p| reach(p) && dist(p.0, p.1, cx, cy) > now + 15_000.0)
                .map(|p| (score(p, None) + ((dist(p.0, p.1, cx, cy) - now) / 1_000.0) as i64
                    + if dist(p.0, p.1, home.0, home.1) < dist(m.x, m.y, home.0, home.1) { 15 } else { 0 }, p))
                .max_by_key(|(sc, p)| (*sc, -p.0, -p.1));
            if let Some((_, to)) = best { return Some(S2Plan::Step(to.0, to.1)); }
        }
        // the risky play (round 55): a low enemy he can finish → step in BEHIND them (between them and their base, cutting
        // the retreat) at gun range and shoot. Riskier than the other steps, but never a dumb one: not when he's low
        // himself, not under an enemy tower (unless the kill is nearly certain and he's healthy), not into 3+ of them.
        if pct(m) >= 35 {
            let (bx, by) = enemy_base(sim, m);
            let (mult, shots) = match self.gun { Gun::Judge => (200, 2), Gun::Operator => (300, 1), Gun::Sheriff => (135, 2),
                Gun::Vandal => (130, 3), Gun::Spectre => (85, 4), Gun::Classic => (100, 3) };
            let burst = m.attack * mult / 100 * shots;
            let rr = match self.gun { Gun::Judge => 9_000, Gun::Operator => 30_000, _ => 16_000 };
            let mut opts: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(STEP_R + rr) && d2(e.x, e.y, m.x, m.y) >= sq(15_000)
                && (pct(e) <= 25 || e.hp <= burst)).collect();
            opts.sort_by_key(|e| (e.hp, e.id));
            for e in opts {
                let back = norm((bx - e.x) as f64, (by - e.y) as f64);
                let sure = e.hp * 2 <= burst && pct(m) >= 60;
                let mut c = ring(m, (e.x, e.y), rr, 16);
                c.extend(ring(m, (e.x, e.y), rr * 2 / 3, 16));
                let best = c.into_iter().map(|p| walls::pull_back(e.x, e.y, p.0, p.1))
                    .filter(|p| reach(p) && ((p.0 - e.x) as f64 * back.0 + (p.1 - e.y) as f64 * back.1) > 0.0)
                    .filter(|p| {
                        let crowd = enemies.iter().filter(|o| o.id != e.id && d2(o.x, o.y, p.0, p.1) <= sq(45_000)).count();
                        let tower = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
                            .any(|t| { let (x, y) = t.pos(); t.is_alive() && t.team() != m.team && d2(x as i64, y as i64, p.0, p.1) <= sq(85_000) });
                        crowd < 3 && (!tower || sure)
                    })
                    .max_by_key(|p| (score(*p, Some(e.id)), -p.0, -p.1));
                if let Some(to) = best { return Some(S2Plan::Step(to.0, to.1)); }
            }
        }
        // inside one of his smokes: the spot in it that is safest
        let in_smoke = |s: &Smoke, toward: (i64, i64)| -> Option<(i64, i64)> {
            let mut c: Vec<(i64, i64)> = into(s, toward).into_iter().collect();
            c.push((s.x, s.y));
            c.extend(ring(m, (s.x, s.y), 15_000, 8));
            c.extend(ring(m, (s.x, s.y), 28_000, 8));
            c.into_iter().filter(|p| inside(s, p.0, p.1) && reach(p)).max_by_key(|p| (score(*p, None), -p.0, -p.1))
        };
        // the ambush step: into one of his smokes that has enemies at it
        if pct(m) >= 50 {
            for s in smokes.iter().filter(|s| !inside(s, m.x, m.y) && d2(s.x, s.y, m.x, m.y) <= sq(STEP_R + SMOKE_R)) {
                let near: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, s.x, s.y) <= sq(SMOKE_R + 18_000)).collect();
                if !near.is_empty() && near.len() <= 3 {
                    if let Some(c) = in_smoke(s, (near[0].x, near[0].y)) { if score(c, None) > -60 { return Some(S2Plan::Step(c.0, c.1)); } }
                }
            }
        }
        // the team's ambush spots (the editor's Map tab): step there when enemies come by it
        if pct(m) >= 50 {
            for k in crate::tactics::marks_for(sim, all, m, crate::tactics::Kind::Ambush) {
                let dd = d2(k.a.0, k.a.1, m.x, m.y);
                if dd <= sq(k.r.min(30_000)) || dd > sq(STEP_R + k.r) || foes_at(k.a.0, k.a.1, k.r + 60_000) == 0 { continue; }
                let mut c = vec![k.a];
                c.extend(ring(m, k.a, k.r / 2, 8));
                c.extend(ring(m, k.a, k.r, 8));
                if let Some(to) = c.into_iter().map(|p| walls::pull_back(m.x, m.y, p.0, p.1))
                    .filter(|p| reach(p) && d2(p.0, p.1, k.a.0, k.a.1) <= sq(k.r)).max_by_key(|p| (score(*p, None), -p.0, -p.1)) {
                    if score(to, None) > -60 { return Some(S2Plan::Step(to.0, to.1)); }
                }
            }
        }
        // Paranoia: any clean line onto an enemy in range; or (round 55, the leader's trade-off) a line that blinds 3+
        // enemies through a teammate or two, when it blinds more than the clean line
        let close = |hits: &Vec<usize>| hits.iter().any(|h| enemies.iter().any(|e| e.id == *h && d2(e.x, e.y, m.x, m.y) <= sq(PARA_RANGE - 15_000)));
        let clean_n = para.as_ref().map_or(0, |p| p.1.len());
        if clean_n < 3 && pct(m) >= 35 {
            if let Some((d, hits, len)) = self.best_para(sim, m, all, true) {
                if hits.len() >= 3 && hits.len() > clean_n && close(&hits) { return Some(S2Plan::Para(d.0, d.1, len)); }
            }
        }
        if let Some((d, hits, len)) = &para {
            if close(hits) { return Some(S2Plan::Para(d.0, d.1, *len)); }
        }
        // the step on an enemy: with a close-range gun on a lone-ish target, or behind a low one; it lands within his gun's
        // reach of them on the safest side (away from their friends and towers), and not at all if every side is unsafe
        let close_gun = m.has("omn_gun3") || m.has("omn_gun2") || m.has("omn_gun0") || m.has("omn_gun1");
        if pct(m) >= 50 {
            let mut opts: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(STEP_R + 22_000) && d2(e.x, e.y, m.x, m.y) >= sq(22_000)
                && foes_at(e.x, e.y, 40_000) <= 2 && (close_gun || pct(e) <= 40)).collect();
            opts.sort_by_key(|e| (pct(e), e.id));
            let rr = if m.has("omn_gun3") { 9_000 } else { 18_000 };
            for e in opts {
                let mut c = ring(m, (e.x, e.y), rr, 12);
                c.extend(ring(m, (e.x, e.y), rr + 8_000, 12));
                if let Some(to) = c.into_iter().map(|p| walls::pull_back(e.x, e.y, p.0, p.1)).filter(|p| reach(p))
                    .max_by_key(|p| (score(*p, Some(e.id)), -p.0, -p.1)) {
                    if score(to, Some(e.id)) >= -30 { return Some(S2Plan::Step(to.0, to.1)); }
                }
            }
        }
        // lurk: into his smoke when enemies are around it
        if pct(m) >= 40 {
            for s in smokes.iter().filter(|s| !inside(s, m.x, m.y)) {
                if foes_at(s.x, s.y, SMOKE_R + 60_000) > 0 {
                    if let Some(c) = in_smoke(s, (m.x, m.y)) { if score(c, None) > -60 { return Some(S2Plan::Step(c.0, c.1)); } }
                }
            }
        }
        // round 56: the flash. Nothing better to do with it off cooldown and an enemy around → step to a better firing
        // spot: one that keeps an enemy in his gun's reach, clearly safer than where he stands (a bush, his smoke, by his
        // team, away from the crowd and towers), not right on top of anyone unless he holds the shotgun
        if pct(m) >= 35 && enemies.iter().any(|e| d2(e.x, e.y, m.x, m.y) <= sq(STEP_R + self.gun.reach() + 20_000)) {
            let gun = self.gun.reach() - 4_000;
            let min_gap = if self.gun == Gun::Judge { 6_000 } else { 15_000 };
            let here = score((m.x, m.y), None);
            let mut c = ring(m, (m.x, m.y), STEP_R, 16);
            c.extend(ring(m, (m.x, m.y), STEP_R * 6 / 10, 16));
            let best = c.into_iter().filter(|p| reach(p))
                .filter(|p| enemies.iter().any(|e| d2(e.x, e.y, p.0, p.1) <= sq(gun)) && !enemies.iter().any(|e| d2(e.x, e.y, p.0, p.1) < sq(min_gap)))
                .map(|p| (score(p, None), p)).max_by_key(|(sc, p)| (*sc, -p.0, -p.1));
            if let Some((sc, to)) = best {
                if sc >= here + 20 || (sc >= here && enemies.iter().all(|e| d2(e.x, e.y, m.x, m.y) > sq(gun))) {
                    return Some(S2Plan::Step(to.0, to.1));
                }
            }
        }
        None
    }

    fn cast_para(&mut self, sim: &mut StableSim<'_>, m: &Champ, d: (f64, f64), range: i64, tick: usize) {
        let range = range.clamp(60_000, PARA_RANGE);
        let (tx, ty) = (m.x + (d.0 * range as f64) as i64, m.y + (d.1 * range as f64) as i64);
        let spec = ProjectileSpawnV1 {
            caster_id: m.id, team: m.team, x: m.x.max(0) as u64, y: m.y.max(0) as u64, radius: PARA_R as u64, speed: PARA_SPEED as u64,
            move_kind: ProjectileMoveKindV1::Linear.code(), target_x: tx.max(0) as u64, target_y: ty.max(0) as u64, penetrate: true,
            attack_type: AttackTypeV1::Skill.code(), casting_target: CastingTargetV1::EnemyChampion.code(),
            ..ProjectileSpawnV1::default()
        };
        sim.spawn_projectile(&vn(m, "paranoia"), &format!("{MOD_ID}:omen_blind"), &spec);
        fx_seen(sim, &vn(m, "para_cast"), m.id, m.x, m.y, 24);
        self.para = Some(Para { x0: m.x, y0: m.y, dx: d.0, dy: d.1, range, t0: tick, hit: Vec::new() });
        sim.entity_remove_buff(m.id, "omn_para_fly");
        sim.add_buff(m.id, &timed("omn_para_fly", (range / PARA_SPEED) as usize + 2));
    }

    // ---------------------------------------------------------------- blind and shadows

    fn velocity(&self, id: usize) -> (f64, f64) {
        self.vel.iter().find(|v| v.0 == id).map_or((0.0, 0.0), |v| (v.3, v.4))
    }

    fn blind(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], victim: &Champ, tick: usize, dur: usize) {
        if victim.id == m.id || victim.has("v1_parry") { return; }
        let until = tick + dur;
        match self.blinded.iter_mut().find(|b| b.0 == victim.id) {
            Some(b) => b.1 = b.1.max(until),
            None => self.blinded.push((victim.id, until)),
        }
        sim.entity_remove_buff(victim.id, "omn_blinded");
        sim.add_buff(victim.id, &timed("omn_blinded", dur));
        // the shadows: one for every enemy of the victim around them, where they are, drifting on
        for e in all.iter().filter(|c| c.team != victim.team && d2(c.x, c.y, victim.x, victim.y) <= sq(DECOY_R)) {
            if self.decoys.iter().any(|d| d.of == e.id) { continue; }
            let sprite = BASE_SPRITES.iter().find(|n| **n == e.name).copied().unwrap_or(SHADOW_SPRITE);
            // round 55: the shadows can't be killed (they last the whole blind): a big pool and an undying buff
            let stat = StatV1 { attack: 0, hp: 100_000, defence: 1_000, magic_resistance: 1_000, move_speed: 1, ..StatV1::default() };
            let atk = UnitAttackV1 { attack_ratio: 0, attack: 0, range: 1, cooltime: 100_000, duration: 10, start_timing: 5, cancelable: true,
                attack_type: AttackTypeV1::BaseAttack.code() };
            if let Some(uid) = sim.spawn_unit(sprite, e.id, e.team, e.x.max(0) as u64, e.y.max(0) as u64, (dur + 6) as u64, &stat, &atk) {
                sim.apply_cc(uid, &CcV1::of_kind(CcKindV1::Bind, (dur + 6) as u64));
                sim.apply_cc(uid, &CcV1::of_kind(CcKindV1::BlockAttack, (dur + 6) as u64));
                sim.apply_cc(uid, &CcV1::of_kind(CcKindV1::BlockSkill, (dur + 6) as u64));
                let mut keep = timed("omn_shadow", dur + 6);
                keep.undying = true;
                sim.add_buff(uid, &keep);
                let (vx, vy) = self.velocity(e.id);
                self.decoys.push(Decoy { id: uid, of: e.id, team: e.team, x: e.x as f64, y: e.y as f64, vx, vy });
                fx_on(sim, &vn(m, "decoy"), m.id, uid, 30);
            }
        }
        self.decoy_until = self.decoy_until.max(until + 6);
    }

    fn run_blinds(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        // the shadows drift and pop
        self.decoys.retain(|d| sim.get_entity(d.id).map_or(false, |e| e.is_alive()));
        if tick >= self.decoy_until { self.decoys.clear(); }
        for d in self.decoys.iter_mut() {
            let (nx, ny) = (d.x + d.vx, d.y + d.vy);
            if !walls::wall_at(nx as i64, ny as i64) && nx > 0.0 && ny > 0.0 && nx < MAP as f64 && ny < MAP as f64 {
                d.x = nx;
                d.y = ny;
                sim.entity_set_pos(d.id, nx as u64, ny as u64);
            } else {
                d.vx = 0.0;
                d.vy = 0.0;
            }
            if tick % 30 == 0 { fx_on(sim, &vn(m, "decoy"), m.id, d.id, 30); }
        }
        // the blinded: attacks only at a shadow
        self.blinded.retain(|b| b.1 > tick);
        if tick % 4 != 0 { return; }
        for &(vid, _) in &self.blinded {
            let Some(v) = all.iter().find(|c| c.id == vid) else { continue };
            let target = self.decoys.iter().filter(|d| d.team != v.team)
                .min_by_key(|d| (d2(d.x as i64, d.y as i64, v.x, v.y), d.id)).map(|d| d.id);
            match target {
                Some(t) => {
                    let mut taunt = CcV1::of_kind(CcKindV1::Taunt, 6);
                    taunt.target = t;
                    sim.apply_cc(vid, &taunt);
                }
                None => sim.apply_cc(vid, &CcV1::of_kind(CcKindV1::BlockAttack, 6)),
            }
        }
    }

    // ---------------------------------------------------------------- ult

    /// Where From the Shadows should take him, or None:
    ///   escape: low and chased with his step down → home;
    ///   a fight his team is in, far from him → behind the enemies (into his smoke there if there is one);
    ///   one of his smokes with enemies at it and a teammate near (or a weak enemy) → into the smoke;
    ///   a lone enemy at 35% HP or less far away while he's healthy → behind them;
    ///   a big objective the enemies are taking with his teammates near → there.
    fn plan_ult(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ], tick: usize, s2_ready: bool) -> Option<(i64, i64)> {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let mates: Vec<&Champ> = all.iter().filter(|c| c.team == m.team && c.id != m.id).collect();
        let foes_at = |x: i64, y: i64, r: i64| enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(r)).count();
        let mates_at = |x: i64, y: i64, r: i64| mates.iter().filter(|a| d2(a.x, a.y, x, y) <= sq(r)).count();
        let smokes: Vec<Smoke> = smokes_up(all, tick).into_iter().filter(|s| s.owner == m.id && s.end > tick + ULT_T + 30).collect();
        if pct(m) <= 25 && foes_at(m.x, m.y, 35_000) > 0 && !s2_ready {
            if let Some(h) = self.spawn { return Some(h); }
        }
        if pct(m) < 45 { return None; }
        // a fight far away
        let fighters: Vec<&&Champ> = mates.iter().filter(|a| foes_at(a.x, a.y, 45_000) > 0).collect();
        if fighters.len() >= 2 {
            let (ax, ay) = { let n = fighters.len() as i64; (fighters.iter().map(|a| a.x).sum::<i64>() / n, fighters.iter().map(|a| a.y).sum::<i64>() / n) };
            if d2(ax, ay, m.x, m.y) >= sq(150_000) {
                let foes: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, ax, ay) <= sq(60_000)).collect();
                if !foes.is_empty() && foes.len() <= fighters.len() + 1 {
                    let (ex, ey) = { let n = foes.len() as i64; (foes.iter().map(|e| e.x).sum::<i64>() / n, foes.iter().map(|e| e.y).sum::<i64>() / n) };
                    let (dx, dy) = norm((ex - ax) as f64, (ey - ay) as f64);
                    return Some(walls::pull_back(ex, ey, clampm(ex + (dx * 22_000.0) as i64), clampm(ey + (dy * 22_000.0) as i64)));
                }
            }
        }
        // round 67 (Rian): a gank anywhere on the map. A teammate in a lane fight his side wins with him there, far
        // away (he can gank bot from top): land behind the enemy, between them and their base, cutting the retreat
        if foes_at(m.x, m.y, 70_000) == 0 {
            if let Some((e, _)) = gank_target(sim, all, m) {
                let under_tower = enemy_tower_near(sim, m, e.x, e.y, 75_000);
                let mate_ok = mates.iter().any(|a| d2(a.x, a.y, e.x, e.y) <= sq(45_000) && pct(a) >= 35);
                if d2(e.x, e.y, m.x, m.y) >= sq(150_000) && mate_ok && (!under_tower || pct(e) <= 30) {
                    let b = enemy_base(sim, m);
                    let (dx, dy) = norm((b.0 - e.x) as f64, (b.1 - e.y) as f64);
                    return Some(walls::pull_back(e.x, e.y, clampm(e.x + (dx * 18_000.0) as i64), clampm(e.y + (dy * 18_000.0) as i64)));
                }
            }
        }
        // the team's ambush spots with enemies at them and a teammate near
        for k in crate::tactics::marks_for(sim, all, m, crate::tactics::Kind::Ambush) {
            if d2(k.a.0, k.a.1, m.x, m.y) < sq(STEP_R + k.r + 30_000) { continue; }
            if foes_at(k.a.0, k.a.1, k.r + 60_000) > 0 && mates_at(k.a.0, k.a.1, k.r + 90_000) >= 1 { return Some(k.a); }
        }
        // into a smoke: the part of it nearest the enemies at it (hidden on arrival)
        for s in &smokes {
            let near = |r: i64| enemies.iter().filter(|e| d2(e.x, e.y, s.x, s.y) <= sq(r)).copied().collect::<Vec<&Champ>>();
            let foes = near(SMOKE_R + 30_000);
            if foes.is_empty() || d2(s.x, s.y, m.x, m.y) < sq(SMOKE_R + 60_000) { continue; }
            let weak = foes.iter().any(|e| pct(e) <= 40);
            let backed = mates_at(s.x, s.y, SMOKE_R + 40_000);
            if (backed >= 1 && foes.len() <= backed + 1) || (weak && foes.len() == 1) {
                let n = foes.len() as i64;
                let (ex, ey) = (foes.iter().map(|e| e.x).sum::<i64>() / n, foes.iter().map(|e| e.y).sum::<i64>() / n);
                let (dx, dy) = norm((ex - s.x) as f64, (ey - s.y) as f64);
                return Some(walls::pull_back(s.x, s.y, s.x + (dx * SMOKE_R as f64 * 0.5) as i64, s.y + (dy * SMOKE_R as f64 * 0.5) as i64));
            }
        }
        // a pick
        if pct(m) >= 60 {
            if let Some(e) = enemies.iter().filter(|e| pct(e) <= 35 && foes_at(e.x, e.y, 45_000) == 1 && d2(e.x, e.y, m.x, m.y) >= sq(100_000))
                .min_by_key(|e| (pct(e), e.id)) {
                let (dx, dy) = norm((e.x - m.x) as f64, (e.y - m.y) as f64);
                return Some(walls::pull_back(e.x, e.y, clampm(e.x + (dx * 14_000.0) as i64), clampm(e.y + (dy * 14_000.0) as i64)));
            }
        }
        // an objective
        if let Some((ox, oy, ours)) = objective(sim, all, m) {
            if !ours && mates_at(ox, oy, 70_000) >= 1 && d2(ox, oy, m.x, m.y) >= sq(120_000) {
                return Some(walls::pull_back(m.x, m.y, ox, oy));
            }
        }
        None
    }
}

/// Where to land the ult around `p` unseen: a bush near it first (the enemies get no warning), else one of his
/// smokes there; otherwise `p` itself, flagged so a smoke first is the second option. (x, y, hidden)
fn hide_spot(sim: &StableSim<'_>, all: &[Champ], m: &Champ, p: (i64, i64), tick: usize) -> (i64, i64, bool) {
    let foes_on = |x: i64, y: i64| all.iter().any(|c| c.team != m.team && d2(c.x, c.y, x, y) <= sq(12_000));
    // Rian's ult spots and ambush spots near it come first
    let mut spots = crate::tactics::marks_for(sim, all, m, crate::tactics::Kind::Ult);
    spots.extend(crate::tactics::marks_for(sim, all, m, crate::tactics::Kind::Ambush));
    if let Some(k) = spots.into_iter().find(|k| d2(k.a.0, k.a.1, p.0, p.1) <= sq(60_000.max(k.r)) && !foes_on(k.a.0, k.a.1)) {
        return (k.a.0, k.a.1, walls::bush_at(k.a.0, k.a.1) || smokes_up(all, tick).iter().any(|s| inside(s, k.a.0, k.a.1)));
    }
    if let Some(b) = walls::bushes_near(p.0, p.1, 40_000).into_iter().find(|b| !foes_on(b.0, b.1)) {
        return (b.0, b.1, true);
    }
    if smokes_up(all, tick).into_iter().any(|s| s.owner == m.id && s.end > tick + ULT_T + 30 && d2(s.x, s.y, p.0, p.1) <= sq(SMOKE_R - 10_000)) {
        return (p.0, p.1, true);
    }
    (p.0, p.1, false)
}

/// A gank (round 67): a visible enemy in a small lane fight (at most 2 of theirs within 50000, no more than 2 within
/// 100000) with one of his teammates on them, where his side (counting Omen if he would join) is at least even. The
/// lowest-HP such enemy, and how many of his teammates are on them.
fn gank_target<'a>(sim: &StableSim<'_>, all: &'a [Champ], m: &Champ) -> Option<(&'a Champ, usize)> {
    let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
    let foes_at = |x: i64, y: i64, r: i64| enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(r)).count();
    enemies.iter().copied()
        .filter_map(|e| {
            let mates = all.iter().filter(|a| a.team == m.team && a.id != m.id && d2(a.x, a.y, e.x, e.y) <= sq(45_000)).count();
            let foes = foes_at(e.x, e.y, 50_000);
            (mates >= 1 && foes <= 2 && foes_at(e.x, e.y, 100_000) <= 2 && mates + 1 >= foes).then_some((e, mates))
        })
        .min_by_key(|(e, _)| (pct(e), e.id))
}

fn enemy_tower_near(sim: &StableSim<'_>, m: &Champ, x: i64, y: i64, r: i64) -> bool {
    (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .any(|t| t.is_alive() && t.team() != m.team && { let (tx, ty) = t.pos(); d2(tx as i64, ty as i64, x, y) <= sq(r) })
}

/// Half an objective pit (round 66): the smoke's edge runs through the pit's centre, the dome covering the half away
/// from his team (their way there: the teammates within 200000 of it, else Omen himself). Enemies on the near half stay
/// in his team's sight; those on the far half are cut off and see nothing out.
fn half_pit(m: &Champ, mates: &[&Champ], ox: i64, oy: i64) -> (i64, i64) {
    let near: Vec<&&Champ> = mates.iter().filter(|a| d2(a.x, a.y, ox, oy) <= sq(200_000)).collect();
    let (fx, fy) = if near.is_empty() { (m.x, m.y) } else {
        let n = near.len() as i64;
        (near.iter().map(|a| a.x).sum::<i64>() / n, near.iter().map(|a| a.y).sum::<i64>() / n)
    };
    let (dx, dy) = norm((ox - fx) as f64, (oy - fy) as f64);
    let off = SMOKE_R as f64 * 0.95;
    walls::pull_back(ox, oy, clampm(ox + (dx * off) as i64), clampm(oy + (dy * off) as i64))
}

/// A big neutral objective with champions at it: (x, y, his team is taking it).
fn objective(sim: &StableSim<'_>, all: &[Champ], m: &Champ) -> Option<(i64, i64, bool)> {
    let teams: Vec<usize> = all.iter().map(|c| c.team).collect();
    let mut best: Option<(usize, (i64, i64, bool))> = None;
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if !e.is_alive() || e.is_champion() || e.is_tower() || teams.contains(&e.team()) || e.hp().1 < 2_500 { continue; }
        let (x, y) = { let (x, y) = e.pos(); (x as i64, y as i64) };
        let ours = all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, x, y) <= sq(40_000)).count();
        let theirs = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id) && d2(c.x, c.y, x, y) <= sq(40_000)).count();
        if ours + theirs == 0 { continue; }
        let hp = e.hp().1;
        if best.map_or(true, |b| hp > b.0) { best = Some((hp, (x, y, ours >= theirs))); }
    }
    best.map(|b| b.1)
}

/// Paranoia's blind by the distance it flew: BLIND_MAX up close, falling evenly to BLIND_MIN at its full range.
fn blind_time(d: f64) -> usize {
    let f = ((d - BLIND_CLOSE as f64) / (PARA_RANGE - BLIND_CLOSE) as f64).clamp(0.0, 1.0);
    BLIND_MAX - ((BLIND_MAX - BLIND_MIN) as f64 * f) as usize
}

/// Points around a centre (n of them, radius r), kept on the map.
fn ring(_m: &Champ, c: (i64, i64), r: i64, n: usize) -> Vec<(i64, i64)> {
    (0..n).map(|i| {
        let a = i as f64 * std::f64::consts::TAU / n as f64;
        (clampm(c.0 + (a.cos() * r as f64) as i64), clampm(c.1 + (a.sin() * r as f64) as i64))
    }).collect()
}

/// How safe a landing spot is for Omen (higher = safer): visible enemies close to it (but not `ignore`, the one he is
/// stepping onto), enemy towers in reach, teammates by it, a bush or one of his smokes over it.
fn spot_score(sim: &StableSim<'_>, all: &[Champ], m: &Champ, smokes: &[Smoke], p: (i64, i64), ignore: Option<usize>) -> i64 {
    let mut s = 0i64;
    for c in all.iter().filter(|c| c.id != m.id && Some(c.id) != ignore) {
        let d = dist(c.x, c.y, p.0, p.1);
        if c.team != m.team {
            if !sim.is_visible(m.team, c.id) { continue; }
            s -= if d < 25_000.0 { 60 } else if d < 45_000.0 { 30 } else if d < 70_000.0 { 10 } else { 0 };
        } else if d < 45_000.0 {
            s += 15;
        }
    }
    for i in 0..sim.tower_count() {
        if let Some(t) = sim.get_entity(sim.tower_id_at(i)) {
            let (x, y) = t.pos();
            if t.is_alive() && t.team() != m.team && d2(x as i64, y as i64, p.0, p.1) <= sq(85_000) { s -= 80; }
        }
    }
    if walls::bush_at(p.0, p.1) { s += 25; }
    if smokes.iter().any(|k| inside(k, p.0, p.1)) { s += 30; }
    s
}

fn enemy_base(sim: &StableSim<'_>, m: &Champ) -> (i64, i64) {
    let pts: Vec<(i64, i64)> = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .filter(|t| t.is_alive() && t.team() != m.team).map(|t| { let (x, y) = t.pos(); (x as i64, y as i64) }).collect();
    if pts.is_empty() { return (MAP - m.x, MAP - m.y); }
    let n = pts.len() as i64;
    (pts.iter().map(|p| p.0).sum::<i64>() / n, pts.iter().map(|p| p.1).sum::<i64>() / n)
}

impl StablePassive for Omen {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize) {
        if !self.started {
            self.started = true;
            self.credits = START_CREDITS;
        }
        let tick = sim.tick();
        if let Some(e) = sim.get_entity(entity) {
            let (x, y) = e.pos();
            self.spawn = Some((x as i64, y as i64));
        }
        self.alive_since = tick;
        self.paid_alive = 0;
        self.last_buy = 0;
        self.step = None;
        self.ult = None;
    }
    fn on_kill(&mut self, sim: &mut StableSim<'_>, _player: usize, _entity: usize, victim: usize) {
        if sim.get_entity(victim).map_or(false, |e| e.is_champion()) { self.earn(KILL_CR); self.last_credit = sim.tick(); }
    }
    fn on_assist(&mut self, sim: &mut StableSim<'_>, _player: usize, _entity: usize) {
        self.earn(ASSIST_CR);
        self.last_credit = sim.tick();
    }
    fn on_dead(&mut self, _sim: &mut StableSim<'_>, _player: usize) {
        // round 57: he keeps his gun (only the armor is lost), so the money goes into upgrades
        self.earn(DEATH_CR);
        self.armor = 0;
        self.armor_amt = 0;
        self.step = None;
        self.ult = None;
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, target: usize, damage: &mut usize) {
        let (Some(me), Some(t)) = (sim.get_entity(entity), sim.get_entity(target)) else { return };
        let ((mx, my), (tx, ty)) = (me.pos(), t.pos());
        let (mx, my, tx, ty) = (mx as i64, my as i64, tx as i64, ty as i64);
        let team = me.team();
        let d = dist(mx, my, tx, ty);
        let name = me.name().unwrap_or_default();
        let m = Champ { id: entity, team, x: mx, y: my, buffs: Vec::new(), stunned: false, pushed: false, hp: 0, max_hp: 0, attack: 0, name };
        // the tracer
        let tag = match self.gun { Gun::Judge => "pellets", Gun::Operator => "tracer_op", _ => "tracer" };
        let spec = ProjectileSpawnV1 {
            caster_id: entity, team, x: mx.max(0) as u64, y: (my - 3_000).max(0) as u64, radius: 1_000, speed: 14_000,
            move_kind: ProjectileMoveKindV1::Linear.code(), target_x: tx.max(0) as u64, target_y: ty.max(0) as u64, penetrate: true,
            casting_target: CastingTargetV1::None.code(), ..ProjectileSpawnV1::default()
        };
        sim.spawn_projectile(&vn(&m, tag), &format!("{MOD_ID}:noop"), &spec);
        if self.gun == Gun::Judge {
            // a shotgun: huge point blank, falling off fast; the spread clips whoever stands by the target
            let k = if d <= 10_000.0 { 220 } else if d >= 20_000.0 { 70 } else { 220 - ((d - 10_000.0) / 10_000.0 * 150.0) as usize };
            let base = *damage;
            *damage = base * k / 100;
            let all = champions(sim);
            for e in all.iter().filter(|c| c.team != team && c.id != target && d2(c.x, c.y, tx, ty) <= sq(9_000)) {
                sim.deal_damage(entity, e.id, base * k * 35 / 10_000, 0, AttackTypeV1::Skill);
                crate::wave_near(sim, entity, e.id, 9_000, base * k * 35 / 10_000, 0);
            }
        } else if self.gun == Gun::Operator {
            fx(sim, &vn(&m, "op_flash"), entity, mx, my - 3_000, 12);
        }
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        if !self.started { self.started = true; self.credits = START_CREDITS; }

        // ---- economy
        let alive = tick.saturating_sub(self.alive_since) / ALIVE_EVERY;
        if alive > self.paid_alive {
            self.earn(ALIVE_CR * (alive - self.paid_alive) as i64);
            self.paid_alive = alive;
        }
        if tick % 20 == 0 {
            let at_spawn = self.spawn.map_or(false, |(sx, sy)| d2(sx, sy, m.x, m.y) <= sq(BUY_R));
            // round 57: out of combat (no enemy he can see within SAFE_BUY_R) he buys where he stands too
            let calm = !all.iter().any(|c| c.team != m.team && sim.is_visible(m.team, c.id) && d2(c.x, c.y, m.x, m.y) <= sq(SAFE_BUY_R));
            if (at_spawn && (self.last_buy == 0 || tick >= self.last_buy + 120)) || (calm && tick >= self.last_buy + 300) {
                self.buy(sim, &m, &all, tick);
            }
        }
        // a share of every kill his team gets
        let n = sim.kill_log_count();
        if self.kills_seen > n { self.kills_seen = n; }
        while self.kills_seen < n {
            if let Some(k) = sim.kill_log_at(self.kills_seen) { if k.killer_team == m.team { self.earn(TEAM_KILL_CR); } }
            self.kills_seen += 1;
        }
        // smoke / blind assists: tag the enemies he blinded or that stand in his smoke (a tiny hit puts him on the game's
        // assist list), and credit him himself if one dies inside the window and the game didn't
        let mine: Vec<Smoke> = smokes_up(&all, tick).into_iter().filter(|s| s.owner == entity).collect();
        for c in all.iter().filter(|c| c.team != m.team) {
            let blinded = self.blinded.iter().any(|b| b.0 == c.id && b.1 > tick);
            let smoked = mine.iter().any(|s| d2(c.x, c.y, s.x, s.y) <= sq(SMOKE_R + 4_000));
            if !(blinded || smoked) { continue; }
            match self.tagged.iter_mut().find(|t| t.0 == c.id) {
                Some(t) => {
                    t.1 = tick + ASSIST_WINDOW;
                    if tick >= t.2 { t.2 = tick + TAG_EVERY; sim.deal_damage(entity, c.id, 0, TAG_AP, AttackTypeV1::Skill); }
                }
                None => {
                    self.tagged.push((c.id, tick + ASSIST_WINDOW, tick + TAG_EVERY));
                    sim.deal_damage(entity, c.id, 0, TAG_AP, AttackTypeV1::Skill);
                }
            }
        }
        let mut died = Vec::new();
        self.tagged.retain(|t| {
            let dead = sim.get_entity(t.0).map_or(true, |e| !e.is_alive());
            if dead && tick <= t.1 { died.push(tick); }
            !dead && tick <= t.1
        });
        self.pending.extend(died);
        let lc = self.last_credit;
        let mut add = 0;
        self.pending.retain(|&d| {
            if tick < d + 3 { return true; }
            if lc + 2 < d { add += 1; }
            false
        });
        for _ in 0..add { self.earn(ASSIST_CR); fx_on(sim, &vn(&m, "buy"), entity, entity, 30); }
        self.show_kit(sim, &m);

        // ---- velocities (for the shadows' drift)
        for c in &all {
            match self.vel.iter_mut().find(|v| v.0 == c.id) {
                Some(v) => {
                    let (dx, dy) = ((c.x - v.1) as f64, (c.y - v.2) as f64);
                    if dx.hypot(dy) < 6_000.0 {
                        v.3 = v.3 * 0.7 + dx * 0.3;
                        v.4 = v.4 * 0.7 + dy * 0.3;
                    } else {
                        v.3 = 0.0;
                        v.4 = 0.0;
                    }
                    v.1 = c.x;
                    v.2 = c.y;
                }
                None => self.vel.push((c.id, c.x, c.y, 0.0, 0.0)),
            }
        }

        let cd = Self::player_cd(sim, player);
        let (s1_ready, s2_ready, ult_ready) = (cd.1 == 0, cd.2 == 0, cd.3 == 0);

        // ---- markers from the data file
        if m.has("omn_s1") && !m.has("omn_s1seen") {
            sim.add_buff(entity, &timed("omn_s1seen", 10));
            if self.smoke_bought && m.has("omn_more") { self.smoke_bought = false; }
            let p = self.smoke_plan(sim, &m, &all, tick, s2_ready).or(self.s1_plan);
            if let Some((x, y)) = p { self.place_smoke(sim, &m, x, y, tick); }
            self.s1_plan = None;
        }
        if m.has("omn_s2") && !m.has("omn_s2seen") && self.step.is_none() {
            sim.add_buff(entity, &timed("omn_s2seen", 10));
            match self.s2_plan(sim, &m, &all, tick).or(self.s2_plan) {
                Some(S2Plan::Para(dx, dy, len)) => self.cast_para(sim, &m, (dx, dy), len, tick),
                Some(S2Plan::Step(x, y)) => {
                    let (x, y) = walls::pull_back(m.x, m.y, x, y);
                    self.step = Some((x, y, tick + STEP_T));
                    fx(sim, &vn(&m, "step_in"), entity, x, y, STEP_T as u64);
                    fx(sim, &vn(&m, "step_out"), entity, m.x, m.y, STEP_T as u64);
                    sim.apply_cc(entity, &CcV1::of_kind(CcKindV1::Bind, STEP_T as u64));
                }
                None => {}
            }
            self.s2_plan = None;
        }
        if m.has("omn_ult") && !m.has("omn_ultseen") && self.ult.is_none() {
            sim.add_buff(entity, &timed("omn_ultseen", 10));
            if let Some((x, y)) = self.plan_ult(sim, &m, &all, tick, s2_ready).map(|p| { let h = hide_spot(sim, &all, &m, p, tick); (h.0, h.1) }).or(self.ult_plan) {
                let (x, y) = walls::pull_back(m.x, m.y, x, y);
                let stat = StatV1 { attack: 0, hp: (m.max_hp / 4).max(150), defence: 15, magic_resistance: 15, move_speed: 1, ..StatV1::default() };
                let atk = UnitAttackV1 { attack_ratio: 0, attack: 0, range: 1, cooltime: 100_000, duration: 10, start_timing: 5, cancelable: true,
                    attack_type: AttackTypeV1::BaseAttack.code() };
                let shade = sim.spawn_unit(SHADOW_SPRITE, entity, m.team, x as u64, y as u64, (ULT_T + 4) as u64, &stat, &atk);
                if let Some(s) = shade {
                    sim.apply_cc(s, &CcV1::of_kind(CcKindV1::Bind, (ULT_T + 4) as u64));
                    sim.apply_cc(s, &CcV1::of_kind(CcKindV1::BlockAttack, (ULT_T + 4) as u64));
                    sim.apply_cc(s, &CcV1::of_kind(CcKindV1::BlockSkill, (ULT_T + 4) as u64));
                }
                fx(sim, &vn(&m, "ult_mark"), entity, x, y, 60);
                fx(sim, &vn(&m, "step_out"), entity, m.x, m.y, 36);
                sim.entity_banish(entity, entity, 12, "", "");
                self.ult = Some(UltCh { x, y, t_end: tick + ULT_T, shade });
            }
            self.ult_plan = None;
        }

        // ---- smokes: expire, redraw
        let before = self.smokes.len();
        self.smokes.retain(|s| s.2 > tick);
        if self.smokes.len() != before { self.publish_smokes(sim, &m, tick); }
        if tick % SMOKE_REDRAW == 0 {
            for &(x, y, end) in &self.smokes {
                let tag = if end <= tick + 60 { "smoke_fade" } else { "smoke_dome" };
                fx_seen(sim, &vn(&m, tag), entity, x, y, 30);
            }
        }

        // ---- Paranoia in flight: enemies are marked by the projectile itself; teammates by its path
        if let Some(p) = self.para.clone() {
            let t = tick.saturating_sub(p.t0) as f64;
            let travelled = (t * PARA_SPEED as f64).min(p.range as f64);
            let (px, py) = (p.x0 + (p.dx * travelled) as i64, p.y0 + (p.dy * travelled) as i64);
            // round 59: a trail of smoke puffs along its path, seen by both teams
            if (tick - p.t0) % 5 == 0 { fx_seen(sim, &vn(&m, "para_trail"), entity, px, py, 30); }
            let mut hit = p.hit.clone();
            let touched: Vec<Champ> = all.iter().filter(|c| c.id != entity && !hit.contains(&c.id))
                .filter(|c| (c.team != m.team && c.has("omn_bhit")) || (c.team == m.team && d2(c.x, c.y, px, py) <= sq(PARA_R + 8_000)))
                .cloned().collect();
            for c in &touched {
                hit.push(c.id);
                let dur = blind_time(dist(p.x0, p.y0, c.x, c.y));
                self.blind(sim, &m, &all, c, tick, dur);
            }
            self.para = if travelled >= p.range as f64 { None } else { Some(Para { hit, ..p }) };
        }
        self.run_blinds(sim, &m, &all, tick);
        // ---- the call (round 55): 2+ enemies blinded at once → Omen calls his team to rush them while they can't see
        //      (Steve's call: teammates within 220000 above 30% HP head there; it lasts until the blinds wear off), only
        //      when his side can take it: as many of his team able to come as enemies there that can still see
        if tick % 10 == 0 && !m.buffs.iter().any(|b| b.name().starts_with("stv_rally:")) {
            let blind: Vec<(&Champ, usize)> = self.blinded.iter().filter_map(|&(id, until)| all.iter().find(|c| c.id == id && c.team != m.team).map(|c| (c, until))).collect();
            if blind.len() >= 2 {
                let n = blind.len() as i64;
                let (cx, cy) = (blind.iter().map(|b| b.0.x).sum::<i64>() / n, blind.iter().map(|b| b.0.y).sum::<i64>() / n);
                let ours = all.iter().filter(|c| c.team == m.team && pct(c) > 30 && d2(c.x, c.y, cx, cy) <= sq(220_000)).count();
                let seeing = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id) && !c.has("omn_blinded")
                    && d2(c.x, c.y, cx, cy) <= sq(70_000)).count();
                let until = blind.iter().map(|b| b.1).min().unwrap_or(tick);
                if ours >= 2 && ours >= seeing && until > tick + 30 {
                    crate::steve::note_map_buff(sim.seed());
                    sim.add_buff(entity, &timed(&format!("stv_rally:{cx}:{cy}:-1:{until}"), until - tick + 2));
                    fx(sim, &vn(&m, "ult_mark"), entity, cx, cy, 20);
                }
            }
        }

        // ---- the step
        if let Some((x, y, t_end)) = self.step {
            if tick >= t_end {
                sim.entity_set_pos(entity, x.max(0) as u64, y.max(0) as u64);
                fx(sim, &vn(&m, "step_pop"), entity, x, y, 24);
                self.step = None;
            }
        }
        // ---- the ult channel: hidden away; the shade killed = cancelled
        if let Some(u) = self.ult.clone() {
            let shade_alive = u.shade.map_or(true, |s| sim.get_entity(s).map_or(false, |e| e.is_alive()));
            if !shade_alive {
                fx(sim, &vn(&m, "step_pop"), entity, u.x, u.y, 24);
                self.ult = None;
            } else if tick + 1 >= u.t_end {
                sim.entity_set_pos(entity, u.x.max(0) as u64, u.y.max(0) as u64);
                fx(sim, &vn(&m, "ult_arrive"), entity, u.x, u.y, 30);
                self.ult = None;
            } else {
                if tick % 10 == 0 { sim.entity_banish(entity, entity, (u.t_end - tick).min(12), "", ""); }
                if tick % 30 == 0 { fx(sim, &vn(&m, "ult_mark"), entity, u.x, u.y, 30); }
            }
        }

        // ---- plans and gates (every 10 ticks); the cast is pressed by the AI at an enemy in CC: a 2-tick hold on the
        //      nearest visible enemy when one of the three has a plan
        if tick % 10 == 0 && self.step.is_none() && self.ult.is_none() && !m.stunned {
            let raw = if ult_ready { self.plan_ult(sim, &m, &all, tick, s2_ready) } else { None };
            let hid = raw.map(|p| hide_spot(sim, &all, &m, p, tick));
            self.ult_plan = hid.map(|h| (h.0, h.1));
            self.ult_smoke = hid.map_or(false, |h| !h.2);
            self.s1_plan = if s1_ready { self.smoke_plan(sim, &m, &all, tick, s2_ready) } else { None };
            // round 59: chased with Paranoia ready, he throws it himself: the game's AI doesn't press skills while it
            // runs. The data's S2 stays shut (no omn_s2_ok) for the skill's cooldown, so it can't fire twice.
            let mut locked = m.has("omn_s2_lock");
            if s2_ready && !locked && self.para.is_none() {
                if let Some((d, len)) = self.retreat_para(sim, &m, &all) {
                    self.cast_para(sim, &m, d, len, tick);
                    sim.add_buff(entity, &timed("omn_s2_lock", S2_LOCK));
                    locked = true;
                } else if let Some((x, y)) = self.travel_step(sim, &m, &all) {
                    // no enemy near, so the game's AI has nobody to press it at: he steps by himself
                    self.step = Some((x, y, tick + STEP_T));
                    fx(sim, &vn(&m, "step_in"), entity, x, y, STEP_T as u64);
                    fx(sim, &vn(&m, "step_out"), entity, m.x, m.y, STEP_T as u64);
                    sim.apply_cc(entity, &CcV1::of_kind(CcKindV1::Bind, STEP_T as u64));
                    sim.add_buff(entity, &timed("omn_s2_lock", S2_LOCK));
                    locked = true;
                }
            }
            self.s2_plan = if s2_ready && !locked { self.s2_plan(sim, &m, &all, tick) } else { None };
            // smoke a destination before ulting into it
            // a smoke first only when nothing hides the arrival (a smoke also warns the enemies)
            let ult_go = self.ult_plan.filter(|p| !(self.ult_smoke && s1_ready && self.s1_plan == Some(*p)));
            for (n, on) in [("omn_s1_ok", self.s1_plan.is_some()), ("omn_s2_ok", self.s2_plan.is_some()), ("omn_ult_ok", ult_go.is_some())] {
                sim.entity_remove_buff(entity, n);
                if on { sim.add_buff(entity, &timed(n, 24)); }
            }
            let any = self.s1_plan.is_some() || self.s2_plan.is_some() || ult_go.is_some();
            // round 56: S2 off cooldown with a plan gets the window every refresh (10 ticks), not every 20
            if any && (tick % ULT_BIND_EVERY == 0 || self.s2_plan.is_some()) {
                let key = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id) && d2(c.x, c.y, m.x, m.y) <= sq(190_000))
                    .min_by_key(|c| (d2(c.x, c.y, m.x, m.y), c.id)).map(|c| c.id);
                if let Some(k) = key { sim.apply_cc(k, &CcV1::of_kind(CcKindV1::Bind, 2)); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn half_pit_edge_through_the_centre() {
        let mk = |id, x, y| Champ { id, team: 0, x, y, buffs: Vec::new(), stunned: false, pushed: false, hp: 100, max_hp: 100, attack: 0, name: String::new() };
        let m = mk(1, 400_000, 672_000);
        let a = mk(2, 600_000, 672_000);
        let (x, y) = half_pit(&m, &[&a], 672_000, 672_000);
        assert_eq!(y, 672_000);
        assert!(x > 672_000 + SMOKE_R * 9 / 10 && x <= 672_000 + SMOKE_R, "{x}");
    }

    use super::*;
    #[test]
    fn disk_segment() {
        assert!(seg_hits_disk(0, 0, 100_000, 0, 50_000, 20_000, 26_000));
        assert!(!seg_hits_disk(0, 0, 100_000, 0, 50_000, 30_000, 26_000));
        assert!(!seg_hits_disk(0, 0, 10_000, 0, 50_000, 0, 26_000));
    }
    #[test]
    fn paranoia_trade_through_a_teammate() {
        let c = |id: usize, team: usize, x: i64| Champ { id, team, x, y: 100_000, buffs: Vec::new(), stunned: false, pushed: false,
            hp: 500, max_hp: 1000, attack: 100, name: String::new() };
        let all = vec![c(1, 0, 0), c(2, 0, 30_000), c(3, 1, 60_000), c(4, 1, 75_000), c(5, 1, 90_000)];
        let vis: Vec<&Champ> = all.iter().filter(|x| x.team == 1).collect();
        let o = Omen::default();
        assert!(o.para_line(&all[0], &all, &vis, (1.0, 0.0), false).is_none());
        let (hits, _, allies) = o.para_line(&all[0], &all, &vis, (1.0, 0.0), true).unwrap();
        assert_eq!((hits.len(), allies), (3, 1));
        // 2 enemies for 1 teammate is not worth it
        let two = vec![c(1, 0, 0), c(2, 0, 30_000), c(3, 1, 60_000), c(4, 1, 75_000)];
        let vis2: Vec<&Champ> = two.iter().filter(|x| x.team == 1).collect();
        assert!(o.para_line(&two[0], &two, &vis2, (1.0, 0.0), true).is_none());
    }
    #[test]
    fn blind_scales_with_distance() {
        assert_eq!(blind_time(0.0), 300);
        assert_eq!(blind_time(20_000.0), 300);
        assert_eq!(blind_time(100_000.0), 240);
        assert_eq!(blind_time(180_000.0), 180);
        assert_eq!(blind_time(400_000.0), 180);
    }
    #[test]
    fn gun_prices() {
        assert_eq!(Gun::Judge.price(), 1850);
        assert!(Gun::Operator.tier() > Gun::Vandal.tier());
    }
}
