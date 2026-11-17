//! Deterministic, read-only visual projection. No combat, movement, targeting or RNG writes.
use crate::basalt_core::{Core, Effect, Flying, Frame, Point};
use crate::{basalt_art_data as a, basalt_data as d};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Cue {
    pub name: String,
    pub at: Point,
    pub ticks: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Visuals {
    pub events: Vec<Cue>,
    pub leases: Vec<Cue>,
}
#[derive(Clone, Debug)]
struct Flight {
    air: bool,
    to: Point,
    end: u64,
    start: u64,
    from: Point,
}
#[derive(Clone, Debug)]
struct Toss {
    to: Point,
    end: u64,
    start: u64,
    from: Point,
}
#[derive(Clone, Debug, Default)]
pub struct Presenter {
    shots: Vec<Flight>,
    ignored: Vec<(bool, Point, u64)>,
    tossed: Vec<Toss>,
    meteors: Vec<(Point, u64)>,
    tether: Option<(Point, u64)>,
    due: BTreeMap<String, u64>,
    tokens: usize,
    token_tick: u64,
    last_charge: Option<(bool, Point, u64)>,
}
fn lerp(from: Point, to: Point, k: f64) -> Point {
    Point::new(
        from.x + ((to.x - from.x) as f64 * k).round() as i64,
        from.y + ((to.y - from.y) as f64 * k).round() as i64,
    )
}
fn heading(a: Point, b: Point, count: i64) -> usize {
    ((((b.y - a.y) as f64).atan2((b.x - a.x) as f64) * count as f64 / std::f64::consts::TAU).round()
        as i64)
        .rem_euclid(count) as usize
}
fn lifted(p: Point, height: i64) -> Point {
    Point::new(p.x, p.y - height)
}
type Lease = (String, Cue, u64);
/// Emission priority: standing walls first, then the ult's rock, meteors and their
/// warnings, then everything else. Walls and ult cues never wait behind the per-tick cap.
fn priority(key: &str) -> u8 {
    if key.starts_with("wall:") {
        0
    } else if key.starts_with("charge") || key.starts_with("shot:") || key.starts_with("meteor:") {
        1
    } else {
        2
    }
}
/// Most standing walls plus ult cues alive at once (5 walls; a charge, its warning, the rock or 3 meteors and its
/// warning, six falling meteors with warnings)
pub const PRIORITY_CUES: usize = 24;
fn add(out: &mut Vec<Lease>, key: String, name: String, at: Point, period: u64) {
    {
        out.push((
            key,
            Cue {
                name,
                at,
                ticks: period,
            },
            period,
        ));
    }
}
/// Clip predictable endings exactly; an interruption can leave at most one short lease.
fn add_until(out: &mut Vec<Lease>, key: String, name: String, at: Point, period: u64, remaining: u64) {
    let ticks = a::lease_ticks(period.min(remaining));
    if ticks > 0 {
        add(out, key, name, at, ticks);
    }
}
fn strip(out: &mut Vec<Lease>, key: &str, from: Point, to: Point) {
    let distance = from.dist(to);
    if distance < 1000.0 {
        return;
    }
    let count = (distance / 96000.0).ceil().clamp(1.0, 16.0) as usize;
    let span = distance / count as f64;
    let length = *a::WHIP_LENGTHS
        .iter()
        .min_by_key(|l| (**l as f64 - span).abs() as i64)
        .unwrap();
    let h = heading(from, to, a::WHIP_HEADINGS);
    for i in 0..count {
        let p = lerp(from, to, (i as f64 + 0.5) / count as f64);
        add(
            out,
            format!("{key}:{i}"),
            format!("bs_whip_{length}_{h}"),
            p,
            a::WHIP_PERIOD as u64,
        );
    }
}
impl Presenter {
    /// Spawn/death forgets positions, not combat state. Existing released core
    /// effects are never reconstructed from a new spawn location.
    pub fn reset(&mut self, core: &Core) {
        *self = Self::default();
        self.ignored = core.boulders.clone();
    }
    fn event_ok(&mut self, tick: u64) -> bool {
        if tick != self.token_tick {
            self.tokens = (self.tokens
                + ((tick.saturating_sub(self.token_tick) / 2) as usize).max(1))
            .min(d::FX_TOKENS);
            self.token_tick = tick;
        }
        if self.tokens == 0 {
            false
        } else {
            self.tokens -= 1;
            true
        }
    }
    pub fn step(&mut self, core: &Core, f: &Frame, effects: &[Effect]) -> Visuals {
        if !f.alive {
            self.reset(core);
            return Visuals::default();
        }
        let mut result = Visuals::default();
        let impact_boulders: Vec<Point> = self
            .shots
            .iter()
            .filter(|s| !s.air && f.tick >= s.end)
            .map(|s| f.nearest_ground(s.to))
            .collect();
        let impact_toss: Vec<Point> = self
            .tossed
            .iter()
            .filter(|s| f.tick >= s.end)
            .map(|s| s.to)
            .collect();
        let mut impact_meteors: Vec<Point> = self
            .meteors
            .iter()
            .filter(|m| f.tick >= m.1)
            .map(|m| m.0)
            .collect();
        impact_meteors.extend(
            self.shots
                .iter()
                .filter(|s| s.air && f.tick >= s.end)
                .map(|s| f.nearest_ground(s.to)),
        );
        let air_release = self.last_charge.is_some_and(|c| c.0 && f.tick >= c.2)
            && core.boulders.iter().any(|s| s.0 && s.2 > f.tick);
        for e in effects {
            if let Effect::Fx { name, at, ticks } = e {
                let tag = if name == "bs_slam" && impact_boulders.contains(at) {
                    "bs_boulder_impact"
                } else if name == "bs_slam" && impact_toss.contains(at) {
                    "bs_hurl_impact"
                } else if name == "bs_slam" {
                    "bs_clap_impact"
                } else if name == "bs_rock" && *at == f.pos && air_release {
                    "bs_punch_impact"
                } else if name == "bs_rock" && impact_meteors.contains(at) {
                    "bs_meteor_impact"
                } else {
                    name.as_str()
                };
                if self.event_ok(f.tick) {
                    result.events.push(Cue {
                        name: a::timed_name(tag, (*ticks).min(24)),
                        at: if tag == "bs_punch_impact" {
                            Point::new(f.pos.x + 55000, f.pos.y - a::AIR_LIFT)
                        } else {
                            *at
                        },
                        ticks: (*ticks).min(24),
                    });
                }
            }
        }
        self.shots
            .retain(|s| core.boulders.contains(&(s.air, s.to, s.end)));
        self.ignored.retain(|s| core.boulders.contains(s));
        for &(air, to, end) in &core.boulders {
            if !self
                .shots
                .iter()
                .any(|s| s.air == air && s.to == to && s.end == end)
                && !self.ignored.contains(&(air, to, end))
            {
                self.shots.push(Flight {
                    air,
                    to,
                    end,
                    start: f.tick,
                    from: if air {
                        Point::new(f.pos.x + 55000, f.pos.y)
                    } else {
                        f.pos
                    },
                });
            }
        }
        self.tossed.retain(|s| {
            core.flying
                .iter()
                .any(|r| r.to == s.to && r.land_at == s.end)
        });
        for Flying { to, land_at, .. } in &core.flying {
            if !self.tossed.iter().any(|s| s.to == *to && s.end == *land_at) {
                self.tossed.push(Toss {
                    to: *to,
                    end: *land_at,
                    start: f.tick,
                    from: effects
                        .iter()
                        .find_map(|e| match e {
                            Effect::Fx { name, at, .. }
                                if name == "bs_dust" || name == "bs_break" =>
                            {
                                Some(*at)
                            }
                            _ => None,
                        })
                        .unwrap_or(f.pos),
                });
            }
        }
        self.meteors = core.meteors.clone();
        self.last_charge = core.charging;
        if let Some(Effect::Glide { ticks, .. }) =
            effects.iter().find(|e| matches!(e, Effect::Glide { .. }))
        {
            // Pull/recast's dash marker is the actual wall anchor. Air Dash's
            // marker is on the caster and does not produce a wall tether.
            self.tether = effects.iter().find_map(|e| match e {
                Effect::Fx { name, at, .. } if name == "bs_dash" && *at != f.pos => {
                    Some((*at, f.tick + ticks))
                }
                _ => None,
            });
        }
        if self.tether.is_some_and(|t| f.tick >= t.1) || !core.airborne(f) {
            self.tether = None;
        }
        if core.charging.is_none()
            && self.shots.is_empty()
            && self.tossed.is_empty()
            && core.meteors.is_empty()
            && self.tether.is_none()
            && core.swing.is_none()
            && core.claps.is_empty()
            && core.walls.is_empty()
        {
            self.due.clear();
            return result;
        }
        let mut list = Vec::with_capacity(32);
        if let Some((air, aim, until)) = core.charging {
            let offset = if air {
                Point::new(f.pos.x + 55000, f.pos.y - a::AIR_LIFT)
            } else {
                lifted(f.pos, a::GROUND_LIFT)
            };
            add_until(
                &mut list,
                "charge".into(),
                "bs_boulder".into(),
                offset,
                a::DYNAMIC_PERIOD as u64,
                until.saturating_sub(f.tick),
            );
            add_until(
                &mut list,
                "charge_warn".into(),
                if air {
                    "bs_meteor_area_warning"
                } else {
                    "bs_boulder_warning"
                }
                .into(),
                f.nearest_ground(aim),
                a::WHIP_PERIOD as u64,
                until.saturating_sub(f.tick),
            );
        }
        for s in &self.shots {
            let landing = f.nearest_ground(s.to);
            let k = (f.tick.saturating_sub(s.start) as f64 / (s.end - s.start).max(1) as f64)
                .clamp(0.0, 1.0);
            let lift = (if s.air { a::AIR_LIFT } else { a::GROUND_LIFT }) as f64 * (1.0 - k)
                + a::ARC as f64 * (std::f64::consts::PI * k).sin();
            let p = lifted(lerp(s.from, landing, k), lift.round() as i64);
            let key = format!("shot:{}:{}:{}", s.to.x, s.to.y, s.end);
            if s.air {
                for i in 0..3 {
                    add_until(
                        &mut list,
                        format!("{key}:{i}"),
                        "bs_meteor".into(),
                        Point::new(p.x + (i - 1) * 9000, p.y + (i % 2) * 7000),
                        a::DYNAMIC_PERIOD as u64,
                        s.end.saturating_sub(f.tick),
                    );
                }
            } else {
                add_until(
                    &mut list,
                    key.clone(),
                    "bs_boulder".into(),
                    p,
                    a::DYNAMIC_PERIOD as u64,
                    s.end.saturating_sub(f.tick),
                );
            }
            add_until(
                &mut list,
                format!("{key}:warn"),
                if s.air {
                    "bs_meteor_area_warning"
                } else {
                    "bs_boulder_warning"
                }
                .into(),
                landing,
                a::WHIP_PERIOD as u64,
                s.end.saturating_sub(f.tick),
            );
        }
        for t in &self.tossed {
            let k = (f.tick.saturating_sub(t.start) as f64 / (t.end - t.start).max(1) as f64)
                .clamp(0.0, 1.0);
            let p = lifted(
                lerp(t.from, t.to, k),
                (32000.0 * (std::f64::consts::PI * k).sin()) as i64,
            );
            add_until(
                &mut list,
                format!("toss:{}", t.end),
                "bs_square".into(),
                p,
                a::DYNAMIC_PERIOD as u64,
                t.end.saturating_sub(f.tick),
            );
            add_until(
                &mut list,
                format!("toss_warn:{}", t.end),
                "bs_hurl_warning".into(),
                t.to,
                a::WHIP_PERIOD as u64,
                t.end.saturating_sub(f.tick),
            );
        }
        for &(p, end) in &core.meteors {
            let key = format!("meteor:{}:{}:{end}", p.x, p.y);
            add_until(
                &mut list,
                key.clone(),
                "bs_meteor".into(),
                lifted(p, end.saturating_sub(f.tick) as i64 * 3000),
                a::DYNAMIC_PERIOD as u64,
                end.saturating_sub(f.tick),
            );
            add_until(
                &mut list,
                key + ":warn",
                "bs_meteor_warning".into(),
                p,
                a::WHIP_PERIOD as u64,
                end.saturating_sub(f.tick),
            );
        }
        let hand = |target: Point| {
            Point::new(
                f.pos.x + if target.x < f.pos.x { -8000 } else { 8000 },
                f.pos.y - 8000 - if core.airborne(f) { 20000 } else { 0 },
            )
        };
        if let Some((anchor, _)) = self.tether {
            strip(&mut list, "pull", hand(anchor), anchor);
        }
        if let Some(s) = &core.swing {
            if let Some(u) = f
                .unit(s.target)
                .filter(|u| u.visible && u.hp > 0 && u.targetable)
            {
                strip(&mut list, "swing", hand(u.pos), u.pos);
            }
        }
        for c in &core.claps {
            let end = Point::new(
                c.apex.x + (c.dir.0 * d::CLAP_REACH as f64) as i64,
                c.apex.y + (c.dir.1 * d::CLAP_REACH as f64) as i64,
            );
            let h = heading(c.apex, end, a::CONE_HEADINGS);
            let (dx, dy) = a::CONE_OFFSETS[h];
            add_until(
                &mut list,
                format!("cone:{}", c.start),
                format!("bs_clap_cone_{h}"),
                Point::new(c.apex.x + dx, c.apex.y + dy),
                a::DYNAMIC_PERIOD as u64,
                (c.start + d::CLAP_CLOSE_TICKS as u64).saturating_sub(f.tick),
            );
            for id in [c.left, c.right] {
                if let Some(w) = core.walls.iter().find(|w| w.id == id) {
                    strip(&mut list, &format!("clap:{id}"), hand(w.mid()), w.mid());
                }
            }
        }
        for w in &core.walls {
            let moving =
                w.slide.is_some() || core.claps.iter().any(|c| c.left == w.id || c.right == w.id);
            add(
                &mut list,
                format!("wall:{}", w.id),
                "bs_square".into(),
                w.mid(),
                if moving {
                    a::DYNAMIC_PERIOD
                } else {
                    a::WALL_PERIOD
                } as u64,
            );
        }
        // Priority first, then oldest due: walls and the ult are never starved by whip strips and cones.
        list.sort_by_key(|l| (priority(&l.0), self.due.get(&l.0).copied().unwrap_or(0)));
        let (mut top, mut rest) = (0usize, 0usize);
        list.retain(|l| {
            if priority(&l.0) < 2 {
                top += 1;
                top <= PRIORITY_CUES
            } else {
                rest += 1;
                rest <= a::MAX_CUES as usize
            }
        });
        self.due.retain(|key, _| list.iter().any(|l| &l.0 == key));
        let mut capped = 0;
        for (key, mut cue, period) in list {
            let top = priority(&key) < 2;
            if !top && capped >= a::MAX_EMISSIONS as usize {
                continue;
            }
            if self.due.get(&key).copied().unwrap_or(0) <= f.tick {
                // A view effect expires at its packed animation's end, NOT at
                // play_view_effect(time). Match that duration to the next renewal:
                // no invisible gaps and no overlapping alpha/old-position copies.
                cue.name = a::timed_name(&cue.name, cue.ticks);
                self.due.insert(key, f.tick + period);
                result.leases.push(cue);
                if !top {
                    capped += 1;
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base(name: &str) -> &str {
        name.rsplit_once("_t").filter(|(_, t)| t.parse::<u64>().is_ok()).map_or(name, |(n, _)| n)
    }
    /// Read the SHIPPED frame durations independently of the generated lifetime table.
    fn packed_ticks(cue: &Cue) -> u64 {
        static ANIMS: std::sync::OnceLock<BTreeMap<String, u64>> = std::sync::OnceLock::new();
        let anims = ANIMS.get_or_init(|| {
            let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/vfx/basalt#anim.fanim")).unwrap();
            data["anims"].as_object().unwrap().iter().map(|(name, anim)| {
                let ticks = anim["frames"].as_array().unwrap().iter().map(|f| f["duration"].as_f64().unwrap() * 60.0).sum::<f64>().round() as u64;
                (format!("bs_{name}"), ticks)
            }).collect()
        });
        let ticks = *anims.get(&cue.name).unwrap_or_else(|| panic!("Unbound {}", cue.name));
        assert_eq!(Some(ticks), a::effect_ticks(&cue.name));
        assert_eq!(ticks, cue.ticks, "SDK time does not stretch {}", cue.name);
        ticks
    }
    fn frame(tick: u64) -> Frame {
        serde_json::from_value(serde_json::json!({"tick":tick,"owner":1,"team":0,"alive":true,"pos":{"x":300000,"y":300000},"hp":2000,"max_hp":2000,"attack":100,"ult_unlocked":true,"units":[],"profile":vec![50;13],"manual":true})).unwrap()
    }
    #[test]
    fn charge_flight_and_impact_follow_the_accepted_clock_without_writing_core() {
        let mut p = Presenter::default();
        let mut c = Core::default();
        c.charging = Some((false, Point::new(600000, 300000), 181));
        let before = serde_json::to_string(&c).unwrap();
        let v = p.step(&c, &frame(1), &[]);
        assert!(v
            .leases
            .iter()
            .any(|x| base(&x.name) == "bs_boulder" && x.at == Point::new(300000, 245000)));
        assert_eq!(serde_json::to_string(&c).unwrap(), before);
        c.charging = None;
        c.boulders.push((false, Point::new(600000, 300000), 221));
        p.step(&c, &frame(181), &[]);
        let v = p.step(&c, &frame(201), &[]);
        assert!(v
            .leases
            .iter()
            .any(|x| base(&x.name) == "bs_boulder" && x.at.x == 450000));
        c.boulders.clear();
        let v = p.step(
            &c,
            &frame(221),
            &[Effect::Fx {
                name: "bs_slam".into(),
                at: Point::new(600000, 300000),
                ticks: 36,
            }],
        );
        assert!(v.events.iter().any(|x| base(&x.name) == "bs_boulder_impact"));
        assert!(!v.leases.iter().any(|x| base(&x.name) == "bs_boulder"));
    }
    #[test]
    fn death_spawn_and_independent_owners_do_not_reconstruct_old_flights() {
        let mut c = Core::default();
        c.boulders.push((false, Point::new(600000, 300000), 100));
        let mut p = Presenter::default();
        let mut other = Presenter::default();
        p.step(&c, &frame(1), &[]);
        assert!(other
            .step(&Core::default(), &frame(2), &[])
            .leases
            .is_empty());
        let mut dead = frame(3);
        dead.alive = false;
        assert_eq!(p.step(&c, &dead, &[]), Visuals::default());
        assert!(p.step(&c, &frame(4), &[]).leases.is_empty());
    }
    #[test]
    fn tether_uses_the_wall_and_turns_off_at_the_actual_motion_end() {
        let mut p = Presenter::default();
        let mut c = Core::default();
        c.air = Some(crate::basalt_core::Air {
            land: Point::new(450000, 300000),
            until: 30,
            double_left: false,
            spent: false,
        });
        let anchor = Point::new(400000, 300000);
        let v = p.step(
            &c,
            &frame(1),
            &[
                Effect::Glide {
                    to: c.air.unwrap().land,
                    ticks: 20,
                },
                Effect::Fx {
                    name: "bs_dash".into(),
                    at: anchor,
                    ticks: 18,
                },
            ],
        );
        assert!(v.leases.iter().any(|x| x.name.starts_with("bs_whip_")));
        assert!(!p
            .step(&c, &frame(21), &[])
            .leases
            .iter()
            .any(|x| x.name.starts_with("bs_whip_")));
    }
    #[test]
    fn emissions_and_leases_remain_bounded_and_rendering_never_calls_rng() {
        let mut p = Presenter::default();
        let mut c = Core::default();
        for i in 0..5 {
            let at = Point::new(350000 + i * 10000, 300000);
            c.walls.push(crate::basalt_core::StoneWall {
                id: i as u32,
                a: at,
                b: at,
                integrity: 3,
                born: 1,
                out_since: None,
                slide: None,
            });
        }
        for t in 1..100 {
            let v = p.step(&c, &frame(t), &[]);
            assert!(v.leases.iter().filter(|c| base(&c.name) != "bs_square").count() <= a::MAX_EMISSIONS as usize);
            assert!(v.leases.len() <= a::MAX_EMISSIONS as usize + PRIORITY_CUES);
            assert!(v.leases.iter().all(|c| packed_ticks(c) <= a::WALL_PERIOD as u64));
            assert!(p.due.len() <= a::MAX_CUES as usize + PRIORITY_CUES);
        }
    }

    /// Every lease name -> the last tick it is covered through
    fn coverage(cover: &mut BTreeMap<String, u64>, v: &Visuals, t: u64, key: impl Fn(&Cue) -> Option<String>) {
        for c in &v.leases {
            if let Some(k) = key(c) {
                let e = cover.entry(k).or_insert(0);
                *e = (*e).max(t + packed_ticks(c) - 1);
            }
        }
    }
    fn busy_core(meteors: bool) -> Core {
        let mut c = Core::default();
        for i in 0..5 {
            let at = Point::new(250000 + i * 40000, 200000);
            c.walls.push(crate::basalt_core::StoneWall { id: i as u32, a: at, b: at, integrity: 3, born: 1, out_since: None, slide: None });
        }
        // a clap (cone + two whip strips) and a swing tether keep the capped cues full
        c.claps.push(crate::basalt_core::Clap { start: 1, apex: Point::new(300000, 300000), dir: (0.0, -1.0), left: 0, right: 1, r0: 0, r1: 0 });
        if meteors {
            for k in 0..6 {
                c.meteors.push((Point::new(500000 + k * 20000, 450000), 10_000 + k as u64));
            }
        }
        c
    }

    #[test]
    fn standing_walls_never_blink_even_with_the_emission_cap_full() {
        let mut p = Presenter::default();
        let c = busy_core(false);
        let mut cover = BTreeMap::new();
        for t in 1..=600u64 {
            let v = p.step(&c, &frame(t), &[]);
            coverage(&mut cover, &v, t, |x| (base(&x.name) == "bs_square").then(|| format!("{:?}", x.at)));
            if t > 1 {
                assert_eq!(cover.len(), 5, "every wall drawn");
                for (k, until) in &cover {
                    assert!(*until >= t, "wall {k} has a gap at tick {t}");
                }
            }
        }
    }

    #[test]
    fn the_ult_never_blinks_while_six_meteors_fall_beside_walls_and_whips() {
        let mut p = Presenter::default();
        let mut c = busy_core(true);
        c.charging = Some((true, Point::new(500000, 450000), 10_000));
        let mut cover = BTreeMap::new();
        for t in 1..=600u64 {
            let v = p.step(&c, &frame(t), &[]);
            coverage(&mut cover, &v, t, |x| {
                (x.name.starts_with("bs_meteor") || x.name.starts_with("bs_boulder") || base(&x.name) == "bs_square").then(|| format!("{}:{:?}", base(&x.name), x.at))
            });
            if t > 1 {
                // the falling meteors move each renewal, so check by name class: something of each kind covers every tick
                for kind in ["bs_meteor:", "bs_meteor_warning:", "bs_meteor_area_warning:", "bs_boulder:", "bs_square:"] {
                    assert!(cover.iter().any(|(k, u)| k.starts_with(kind) && *u >= t), "{kind} gap at tick {t}");
                }
                let warnings = cover.iter().filter(|(k, u)| k.starts_with("bs_meteor_warning:") && **u >= t).count();
                assert_eq!(warnings, 6, "every meteor warning drawn at tick {t}");
            }
        }
    }

    #[test]
    fn packed_leases_have_exactly_one_copy_on_every_frame_even_at_fractional_ticks() {
        let mut p = Presenter::default();
        let mut c = busy_core(true);
        c.charging = Some((true, Point::new(500000, 450000), 1000));
        let mut active: Vec<(Cue, u64)> = vec![];
        for tick in 1..=600 {
            active.retain(|(cue, born)| tick < born + packed_ticks(cue));
            let v = p.step(&c, &frame(tick), &[]);
            for cue in v.leases { packed_ticks(&cue); active.push((cue, tick)); }
            for fraction in [0.0, 0.25, 0.5, 0.999] {
                let visible: Vec<_> = active.iter().filter(|(cue, born)| (tick as f64 + fraction) < (*born + packed_ticks(cue)) as f64).collect();
                for wall in &c.walls {
                    assert_eq!(visible.iter().filter(|(cue, _)| base(&cue.name) == "bs_square" && cue.at == wall.mid()).count(), 1, "wall gap or doubled alpha at {tick}+{fraction}");
                }
                assert_eq!(visible.iter().filter(|(cue, _)| base(&cue.name) == "bs_boulder").count(), 1);
                assert_eq!(visible.iter().filter(|(cue, _)| base(&cue.name) == "bs_meteor").count(), 6, "stale moving meteor copies at {tick}");
            }
        }
    }

    #[test]
    fn charge_and_flight_clip_to_the_exact_end_without_a_landing_tail() {
        for end in 30..=36 {
            let mut p = Presenter::default();
            let mut c = Core::default();
            let to = Point::new(600000, 300000);
            let mut active: Vec<(Cue, u64)> = vec![];
            for tick in 1..=end+7 {
                active.retain(|(cue, born)| tick < born + packed_ticks(cue));
                c.charging = (tick < 8).then_some((false, to, 8));
                c.boulders = if (8..end).contains(&tick) { vec![(false, to, end)] } else { vec![] };
                let v = p.step(&c, &frame(tick), &[]);
                for cue in v.leases { active.push((cue, tick)); }
                assert_eq!(active.iter().filter(|(cue, _)| base(&cue.name) == "bs_boulder").count(), usize::from(tick < end), "rock discontinuity at {tick}, end {end}");
                assert_eq!(active.iter().filter(|(cue, _)| base(&cue.name) == "bs_boulder_warning").count(), usize::from(tick < end));
            }
        }
    }

    #[test]
    fn one_shot_events_use_every_existing_drawing_once_with_their_real_lifetime() {
        let mut p = Presenter::default();
        let core = Core::default();
        for (i, name) in ["bs_slam", "bs_break", "bs_land", "bs_wall_slam", "bs_dash"].iter().enumerate() {
            let tick = 1 + i as u64 * 60;
            let v = p.step(&core, &frame(tick), &[Effect::Fx { name: (*name).into(), at: frame(tick).pos, ticks: 18 }]);
            assert_eq!(v.events.len(), 1);
            assert_eq!(packed_ticks(&v.events[0]), 18);
            for t in tick+1..tick+30 { assert!(p.step(&core, &frame(t), &[]).events.is_empty()); }
        }
    }

    #[test]
    fn tossed_squares_cover_every_landing_remainder_without_unbound_short_aliases() {
        for end in 30..=36 {
            let mut p=Presenter::default();let mut c=Core::default();
            let mut active:Vec<(Cue,u64)>=vec![];
            for tick in 1..=end+6 {
                active.retain(|(cue,born)|tick < born+packed_ticks(cue));
                c.flying=if tick<end {vec![Flying{kind:crate::basalt_core::Throw::Hurl,to:Point::new(600000,300000),dir:(1.0,0.0),land_at:end,integrity:3}]} else {vec![]};
                for cue in p.step(&c,&frame(tick),&[]).leases {packed_ticks(&cue);active.push((cue,tick));}
                assert_eq!(active.iter().filter(|(cue,_)|base(&cue.name)=="bs_square").count(),usize::from(tick<end));
            }
        }
    }
}
