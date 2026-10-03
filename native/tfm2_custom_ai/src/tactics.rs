//! Rian's map plans from the editor's Map tab, per team. The editor writes `mods/tfm2_custom_ai/tactics.txt` (one
//! mark per line, already resolved per team and mirrored for the other side when asked); the map customizer reloads
//! it at every match creation, so an edit takes effect from the next match (no career sync needed).
//!
//! Team plan (the strategy: what the team wants; whoever has the tools does it):
//!   block    a choke (line a-b): Steve walls along it, Omen smokes its middle
//!   control  an area: Steve walls it off from the enemy side, Omen smokes the enemies' way in, teammates gather in it
//!   cover    an area: Omen smokes it over (the team works hidden inside: an objective, a push)
//!   fake     a spot: Omen smokes it to deceive (no one needs to be there)
//!   ambush   a spot: Omen lurks / steps / ults there; assigned teammates hide there until enemies come
//!   flank    a spot: assigned teammates go through it before joining a fight near it
//!   avoid    an area: no teammate walks into it
//!   group    a spot: assigned teammates gather there and hold
//!   safe     a spot: teammates at 35% HP or less fall back to it
//! Per-skill marks (advanced): smoke (Omen), wall (Steve), ult (Omen).
//! A tool use (a smoke, a wall) only happens with the team there (someone of his team near the spot), except fakes.
//!
//! Line format (whitespace-separated; '#' starts a comment):
//!   <kind> <champ> <trigger> <team> <priority> <ax> <ay> <bx> <by> [r] [who]
//!   champ    team (team plan), or a champion id suffix / any (per-skill)
//!   trigger  see `active`
//!   team     the team index the mark is for (0 / 1); priority 1..5 (5 first)
//!   r        radius for areas (default 60000); who: all | jungle | mid | top | bot | support | near1 | near2 | omen

use super::*;
use std::sync::RwLock;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind { Smoke, Wall, Ult, Block, Control, Cover, Fake, Ambush, Flank, Avoid, Group, Safe }

#[derive(Clone, Debug)]
pub struct Mark { pub kind: Kind, pub champ: String, pub trigger: String, pub team: usize, pub prio: i64, pub a: (i64, i64), pub b: (i64, i64),
    pub r: i64, pub who: String, pub comp: Vec<String> }

/// The map / vision altering champions (round 49): a plan can be for a team composition of them ("comp=omen+steve"),
/// used only when the team's line-up of these champions is exactly that. Add the next one here.
pub const MAP_CHAMPS: &[&str] = &["omen", "steve"];

/// Which map-altering champions a team has (alive or not), sorted.
pub fn team_comp(sim: &StableSim<'_>, all: &[Champ], team: usize) -> Vec<String> {
    let mut names: Vec<String> = all.iter().filter(|c| c.team == team).map(|c| c.name.clone()).collect();
    for i in 0..sim.player_count() {
        let Some(p) = sim.player_at(i) else { continue };
        if p.team() != team { continue; }
        if let Some(n) = p.champion().and_then(|c| c.name()) { names.push(n); }
    }
    let mut out: Vec<String> = MAP_CHAMPS.iter().filter(|k| names.iter().any(|n| n.ends_with(&format!("_{k}")) || n == *k)).map(|k| k.to_string()).collect();
    out.sort();
    out
}

static MARKS: RwLock<Vec<Mark>> = RwLock::new(Vec::new());

pub fn parse(text: &str) -> Vec<Mark> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 9 { continue; }
        let kind = match f[0] { "smoke" => Kind::Smoke, "wall" => Kind::Wall, "ult" => Kind::Ult, "block" => Kind::Block, "control" => Kind::Control,
            "cover" => Kind::Cover, "fake" => Kind::Fake, "ambush" => Kind::Ambush, "flank" => Kind::Flank, "avoid" => Kind::Avoid,
            "group" => Kind::Group, "safe" => Kind::Safe, _ => continue };
        let r = f.get(9).and_then(|v| v.parse::<f64>().ok()).filter(|v| *v > 0.0).map_or(60_000, |v| (v as i64).clamp(10_000, 300_000));
        let who = f.get(10).map_or("all".to_string(), |w| w.to_lowercase());
        let mut comp: Vec<String> = f.get(11).and_then(|c| c.strip_prefix("comp=")).map_or(Vec::new(), |c| c.split('+').filter(|x| !x.is_empty() && *x != "-").map(|x| x.to_lowercase()).collect());
        comp.sort();
        let n: Vec<i64> = f[3..9].iter().filter_map(|s| s.parse::<f64>().ok().map(|v| v as i64)).collect();
        if n.len() < 6 { continue; }
        out.push(Mark { kind, champ: f[1].to_lowercase(), trigger: f[2].to_lowercase(), team: n[0].max(0) as usize, prio: n[1].clamp(1, 5),
            a: (n[2], n[3]), b: (n[4], n[5]), r, who, comp });
    }
    out
}

/// Reload tactics.txt (next to the DLL's mod folder). Missing file = no marks.
pub fn load() {
    let text = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|d| d.join("mods").join(MOD_ID).join("tactics.txt")))
        .and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
    if let Ok(mut g) = MARKS.write() { *g = parse(&text); }
}

fn for_champ(mark: &Mark, name: &str) -> bool {
    mark.champ == "any" || mark.champ == "team" || name.ends_with(&format!("_{}", mark.champ)) || name == mark.champ
}

fn is_area(k: &Kind) -> bool {
    matches!(k, Kind::Control | Kind::Cover | Kind::Ambush | Kind::Avoid | Kind::Group | Kind::Safe | Kind::Flank | Kind::Fake)
}

/// The big neutral objectives that are up: (x, y, max hp). Morgard (the epic monster) has 8000+ max HP, the serpent less.
fn objectives(sim: &StableSim<'_>, all: &[Champ]) -> Vec<(i64, i64, usize)> {
    let teams: Vec<usize> = all.iter().map(|c| c.team).collect();
    let mut out = Vec::new();
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if !e.is_alive() || e.is_champion() || e.is_tower() || teams.contains(&e.team()) { continue; }
        let mx = e.hp().1;
        if mx >= 4_000 { let (x, y) = e.pos(); out.push((x as i64, y as i64, mx)); }
    }
    out
}

fn sq_(v: i64) -> i128 { (v as i128) * (v as i128) }
fn d2_(a: (i64, i64), b: (i64, i64)) -> i128 { sq_(a.0 - b.0) + sq_(a.1 - b.1) }

/// Kills in the last 15 s: (by `team`, by the others).
fn recent_kills(sim: &StableSim<'_>, team: usize) -> (usize, usize) {
    let now = sim.tick();
    let (mut ours, mut theirs) = (0, 0);
    for i in (0..sim.kill_log_count()).rev() {
        let Some(k) = sim.kill_log_at(i) else { continue };
        if k.tick + 900 < now { break; }
        if k.killer_team == team { ours += 1 } else { theirs += 1 }
    }
    (ours, theirs)
}

/// Champions alive: (on `team`, on the others).
fn alive(sim: &StableSim<'_>, team: usize) -> (usize, usize) {
    let (mut ours, mut theirs) = (0, 0);
    for i in 0..sim.player_count() {
        let Some(p) = sim.player_at(i) else { continue };
        if !p.is_alive() { continue; }
        if p.team() == team { ours += 1 } else { theirs += 1 }
    }
    (ours, theirs)
}

/// Is the mark's trigger on for `team` right now? ("here" = within the mark's area, or 70000-90000 of a spot / line)
///   always · early (first 8 min) · midgame (8-18 min) · late (18 min+)
///   serpen / epic        that objective is up with a champion near it (120000)   [epic = Morgard]
///   serpen_up / epic_up  that objective is up (anywhere, anyone)
///   serpen_fight / epic_fight  it's up and both teams are near it (contested)
///   fight      champions of both teams here           teamfight  3+ of each here
///   defend     2+ enemies here                         attack     2+ of us here
///   won        we won a fight: 2+ kills in the last 15 s, more than theirs
///   lost       we lost one (the reverse)
///   ahead / behind      more / fewer of us alive than of them
///   gank       a lone enemy here (no other enemy within 50000 of them) and 2+ of us within 200000
///   coming     enemies on their way: seen just outside (up to 150000 past the edge) while we're here
///   isolate    (lines) a fight on one side of the line, more enemies on the other side and none of us there
///   won_serpen / won_epic      won a fight and that objective is up
///   ahead_serpen / ahead_epic  more of us alive and that objective is up
///   behind_serpen / behind_epic  fewer of us alive and that objective is up
///   tower      one of our towers here has an enemy champion at it
pub fn active(sim: &StableSim<'_>, all: &[Champ], mark: &Mark, team: usize) -> bool {
    let p = centre(mark);
    let ar = if is_area(&mark.kind) { mark.r + 20_000 } else { 0 };
    let seen = |c: &Champ| c.team == team || sim.is_visible(team, c.id);
    let near = |ally: bool, r: i64| { let r = r.max(ar); all.iter().filter(|c| (c.team == team) == ally && seen(c) && d2_((c.x, c.y), p) <= sq_(r)).count() };
    let objs = || objectives(sim, all);
    let obj = |epic: bool| objs().into_iter().find(|o| (o.2 >= 8_000) == epic);
    let tick = sim.tick();
    match mark.trigger.as_str() {
        "always" => true,
        "early" => tick < 28_800,
        "midgame" => (28_800..64_800).contains(&tick),
        "late" => tick >= 64_800,
        "serpen" | "epic" => obj(mark.trigger == "epic").map_or(false, |o| all.iter().any(|c| d2_((c.x, c.y), (o.0, o.1)) <= sq_(120_000))),
        "serpen_up" | "epic_up" => obj(mark.trigger == "epic_up").is_some(),
        "serpen_fight" | "epic_fight" => obj(mark.trigger == "epic_fight").map_or(false, |o| {
            let side = |ally: bool| all.iter().any(|c| (c.team == team) == ally && seen(c) && d2_((c.x, c.y), (o.0, o.1)) <= sq_(120_000));
            side(true) && side(false)
        }),
        "fight" => near(true, 70_000) > 0 && near(false, 70_000) > 0,
        "teamfight" => near(true, 90_000) >= 3 && near(false, 90_000) >= 3,
        "defend" => near(false, 90_000) >= 2,
        "attack" => near(true, 90_000) >= 2,
        "won" => { let (o, t) = recent_kills(sim, team); o >= 2 && o > t }
        // won a fight and that objective is up: the time to take it (forced takes don't work)
        "won_serpen" | "won_epic" => { let (o, t) = recent_kills(sim, team); o >= 2 && o > t && obj(mark.trigger == "won_epic").is_some() }
        // down in numbers and that objective is up: don't force it
        "behind_serpen" | "behind_epic" => { let (o, t) = alive(sim, team); o < t && obj(mark.trigger == "behind_epic").is_some() }
        // up in numbers and that objective is up
        "ahead_serpen" | "ahead_epic" => { let (o, t) = alive(sim, team); o > t && obj(mark.trigger == "ahead_epic").is_some() }
        "lost" => { let (o, t) = recent_kills(sim, team); t >= 2 && t > o }
        "ahead" => { let (o, t) = alive(sim, team); o > t }
        "behind" => { let (o, t) = alive(sim, team); o < t }
        "gank" => {
            let r = ar.max(60_000);
            let lone = all.iter().filter(|e| e.team != team && seen(e) && d2_((e.x, e.y), p) <= sq_(r))
                .any(|e| !all.iter().any(|o| o.team != team && o.id != e.id && d2_((o.x, o.y), (e.x, e.y)) <= sq_(50_000)));
            lone && all.iter().filter(|c| c.team == team && d2_((c.x, c.y), p) <= sq_(200_000)).count() >= 2
        }
        "coming" => {
            let r = ar.max(40_000);
            let out = all.iter().any(|e| e.team != team && seen(e) && { let d = d2_((e.x, e.y), p); d > sq_(r) && d <= sq_(r + 150_000) });
            out && all.iter().any(|c| c.team == team && d2_((c.x, c.y), p) <= sq_(r + 60_000))
        }
        "isolate" => {
            // a fight on one side of the line (his team and theirs both there), more enemies coming on the other side,
            // and none of his team over there (the wall would cut them off)
            let (a, b) = (mark.a, mark.b);
            let side = |x: i64, y: i64| ((b.0 - a.0) as i128 * (y - a.1) as i128 - (b.1 - a.1) as i128 * (x - a.0) as i128).signum();
            [1i128, -1].iter().any(|&s| {
                let ours = |sd: i128, r: i64| all.iter().filter(|c| c.team == team && side(c.x, c.y) == sd && d2_((c.x, c.y), p) <= sq_(r)).count();
                let theirs = |sd: i128, r: i64| all.iter().filter(|e| e.team != team && seen(e) && side(e.x, e.y) == sd && d2_((e.x, e.y), p) <= sq_(r)).count();
                ours(s, 140_000) > 0 && theirs(s, 140_000) > 0 && theirs(-s, 200_000) > 0 && ours(-s, 200_000) == 0
            })
        }
        "tower" => {
            let r = ar.max(60_000) + 40_000;
            (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i))).filter(|t| t.is_alive() && t.team() == team)
                .any(|t| { let (x, y) = t.pos(); let tp = (x as i64, y as i64);
                    d2_(tp, p) <= sq_(r) && all.iter().any(|e| e.team != team && d2_((e.x, e.y), tp) <= sq_(30_000)) })
        }
        _ => false,
    }
}

/// Where a mark is: the middle of a line, else its point.
pub fn centre(mark: &Mark) -> (i64, i64) {
    if matches!(mark.kind, Kind::Wall | Kind::Block) { ((mark.a.0 + mark.b.0) / 2, (mark.a.1 + mark.b.1) / 2) } else { mark.a }
}

/// Is his team there to use a tool on this mark? Someone of the team (him included) within the mark's reach:
/// its radius + 80000 for areas, 120000 for lines and spots. A fake smoke needs nobody there.
pub fn team_there(all: &[Champ], mark: &Mark, team: usize) -> bool {
    if mark.kind == Kind::Fake { return true; }
    let p = centre(mark);
    let reach = if is_area(&mark.kind) { mark.r + 80_000 } else { 120_000 };
    all.iter().any(|c| c.team == team && d2_((c.x, c.y), p) <= sq_(reach))
}

/// The marks of `kind` for this champion's team that are on now, highest priority first (then nearest).
pub fn marks_for(sim: &StableSim<'_>, all: &[Champ], m: &Champ, kind: Kind) -> Vec<Mark> {
    let list: Vec<Mark> = MARKS.read().map(|g| g.iter().filter(|k| k.kind == kind && k.team == m.team && for_champ(k, &m.name)).cloned().collect())
        .unwrap_or_default();
    // a plan made for a team composition only counts when the team has exactly those map-altering champions
    let comp = if list.iter().any(|k| !k.comp.is_empty()) { team_comp(sim, all, m.team) } else { Vec::new() };
    let mut on: Vec<Mark> = list.into_iter().filter(|k| (k.comp.is_empty() || k.comp == comp) && active(sim, all, k, m.team)).collect();
    on.sort_by_key(|k| (-k.prio, d2_(k.a, (m.x, m.y))));
    on
}

/// Is `me` one of the teammates a move plan is for? (by role, or the nearest 1 / 2 of the team to the spot)
fn assigned(k: &Mark, all: &[Champ], me: &Champ, lane: Option<mod_api_stable::LaneV1>) -> bool {
    use mod_api_stable::LaneV1 as L;
    match k.who.as_str() {
        "all" | "" => true,
        "jungle" => lane == Some(L::Jungle),
        "mid" => lane == Some(L::Mid),
        "top" => lane == Some(L::Top),
        "bot" => lane == Some(L::Bottom),
        "support" => lane == Some(L::Support),
        "omen" => me.name.ends_with("_omen"),
        "near1" | "near2" => {
            let n = if k.who == "near1" { 1 } else { 2 };
            let p = centre(k);
            let mut mates: Vec<&Champ> = all.iter().filter(|c| c.team == me.team && c.max_hp > 0 && c.hp * 100 > c.max_hp * 40).collect();
            mates.sort_by_key(|c| (d2_((c.x, c.y), p), c.id));
            mates.iter().take(n).any(|c| c.id == me.id)
        }
        _ => true,
    }
}

/// The input AI (moves only): the team's move plans (the editor's Map tab). In order:
///   avoid    a move into an avoid area → to just outside it;
///   safe     at 35% HP or less → to the safe spot (within 300000);
///   flank    (assigned) not yet through the flank spot and a fight near it → via the spot first;
///   ambush   (assigned, not Omen: his kit does it) → hide there; held until an enemy comes within the area;
///   group / control  (assigned) within 250000 → there (a group point holds; a controlled area leaves him free inside).
/// Not when he's at 40% HP or less (except avoid and safe).
pub fn move_goal(sim: &StableSim<'_>, all: &[Champ], me: &Champ, lane: Option<mod_api_stable::LaneV1>, dest: (i64, i64)) -> Option<(i64, i64)> {
    let unit = |dx: f64, dy: f64| { let l = dx.hypot(dy); if l < 1e-9 { (1.0, 0.0) } else { (dx / l, dy / l) } };
    let go = |p: (i64, i64)| Some(walls::pull_back(me.x, me.y, p.0.clamp(0, 960_000), p.1.clamp(0, 960_000)));
    let here = (me.x, me.y);
    for k in marks_for(sim, all, me, Kind::Avoid) {
        if d2_(dest, k.a) <= sq_(k.r) {
            let (ux, uy) = if d2_(dest, k.a) > sq_(2_000) { unit((dest.0 - k.a.0) as f64, (dest.1 - k.a.1) as f64) } else { unit((me.x - k.a.0) as f64, (me.y - k.a.1) as f64) };
            return go((k.a.0 + (ux * (k.r + 8_000) as f64) as i64, k.a.1 + (uy * (k.r + 8_000) as f64) as i64));
        }
    }
    let low = me.max_hp == 0 || me.hp * 100 <= me.max_hp * 40;
    if me.max_hp > 0 && me.hp * 100 <= me.max_hp * 35 {
        if let Some(k) = marks_for(sim, all, me, Kind::Safe).into_iter().find(|k| d2_(here, k.a) <= sq_(300_000) && d2_(dest, k.a) > sq_(k.r)) {
            return go(k.a);
        }
    }
    if low { return None; }
    for k in marks_for(sim, all, me, Kind::Flank) {
        if !assigned(&k, all, me, lane) || me.has(&format!("tfl:{}:{}", k.a.0, k.a.1)) || d2_(here, k.a) > sq_(300_000) { continue; }
        // a fight near the flank spot (within its radius + 150000) that he isn't in yet
        let fight = all.iter().any(|c| c.team == me.team && c.id != me.id && d2_((c.x, c.y), k.a) <= sq_(k.r + 150_000)
            && all.iter().any(|e| e.team != me.team && d2_((e.x, e.y), (c.x, c.y)) <= sq_(60_000)));
        let in_it = all.iter().any(|e| e.team != me.team && d2_((e.x, e.y), here) <= sq_(40_000));
        if fight && !in_it { return go(k.a); }
    }
    if !me.name.ends_with("_omen") {
        for k in marks_for(sim, all, me, Kind::Ambush) {
            if !assigned(&k, all, me, lane) || d2_(here, k.a) > sq_(250_000) { continue; }
            if all.iter().any(|e| e.team != me.team && d2_((e.x, e.y), k.a) <= sq_(k.r)) { continue; }   // they came: spring it
            if d2_(here, k.a) <= sq_((k.r / 3).max(12_000)) { return Some(here); }
            return go(k.a);
        }
    }
    let mut gather = marks_for(sim, all, me, Kind::Group);
    gather.extend(marks_for(sim, all, me, Kind::Control));
    for k in gather {
        if !assigned(&k, all, me, lane) { continue; }
        let inner = (k.r / 2).max(15_000);
        if d2_(here, k.a) > sq_(250_000) || d2_(dest, k.a) <= sq_(inner) { continue; }
        if k.kind == Kind::Control && d2_(here, k.a) <= sq_(k.r) { continue; }
        if d2_(here, k.a) <= sq_(inner) { return Some(here); }
        return go(k.a);
    }
    None
}

/// Match hook, every 10 ticks: a teammate who reaches a flank spot is through it ("tfl:x:y" for 15 s).
pub fn tick(sim: &mut StableSim<'_>, all: &[Champ]) {
    let marks: Vec<Mark> = MARKS.read().map(|g| g.iter().filter(|k| k.kind == Kind::Flank).cloned().collect()).unwrap_or_default();
    for k in &marks {
        for c in all.iter().filter(|c| c.team == k.team && d2_((c.x, c.y), k.a) <= sq_(k.r.min(30_000).max(15_000))) {
            let n = format!("tfl:{}:{}", k.a.0, k.a.1);
            if !c.has(&n) { sim.add_buff(c.id, &BuffV1::timed(&n, 900)); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_lines() {
        let m = parse("# comment\nsmoke omen serpen 0 5 400000 300000 0 0\nwall steve epic 1 3 1 2 3 4\nbad line\n");
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].kind, Kind::Smoke);
        assert_eq!(m[1].team, 1);
        assert_eq!(m[1].b, (3, 4));
        let g = parse("control team epic 0 4 100 200 100 200 80000 jungle\navoid team always 1 3 1 1 1 1\nflank team fight 0 2 5 5 5 5 0 near2\n");
        assert_eq!(g[0].kind, Kind::Control);
        assert_eq!(g[0].r, 80_000);
        assert_eq!(g[0].who, "jungle");
        assert_eq!(g[1].r, 60_000);
        assert_eq!(g[1].who, "all");
        assert_eq!(g[2].kind, Kind::Flank);
        assert_eq!(g[2].r, 60_000);
        let c = parse("smoke omen serpen 0 3 1 1 1 1 0 fake comp=steve+omen\nwall steve isolate 1 4 1 1 9 9 0 all comp=steve\n");
        assert_eq!(c[0].comp, vec!["omen".to_string(), "steve".to_string()]);
        assert_eq!(c[0].who, "fake");
        assert_eq!(c[1].comp, vec!["steve".to_string()]);
    }
}
