//! Bounded v1 transport. Native and WASM return complete state, including private event counters.
use crate::gojo_v2_core::*;
pub struct Frame {
    pub selected: usize,
    pub aim: (i64, i64),
    pub tick: usize,
    pub owner: usize,
    pub automatic: bool,
    pub action: Option<Action>,
    pub unlocked: [bool; 3],
    pub controlled: bool,
    pub bound: bool,
    pub basic: Vec<(u64, usize)>,
    pub hit: bool,
    pub shield: Option<i64>,
    pub all: Vec<Fighter>,
    pub walls: Vec<(i64, i64)>,
    pub haste: i64,
    pub ult_haste: i64,
    pub barriers: Vec<(i64, i64, i64)>,
    pub profile: crate::athlete_ai_core::Profile,
    pub scene: crate::athlete_ai_core::Scene,
}
impl Frame {
    pub fn decode(a: &[i64]) -> Option<Self> {
        if a.len() < 17
            || ![3, 4].contains(&a[0])
            || a[1] < 0
            || a[2] < 0
            || !(1..=512).contains(&a[15])
        {
            return None;
        }
        let end = 16 + 15 * a[15] as usize;
        if a.len() <= end
            || !(0..=3600).contains(&a[end])
            || a.len() < end + 1 + 2 * a[end] as usize
        {
            return None;
        }
        let extra = end + 1 + 2 * a[end] as usize;
        let (haste, ult_haste, barriers) = if a.len() == extra {
            (0, 0, vec![])
        } else {
            if a.len() < extra + 3
                || !(0..=64).contains(&a[extra + 2])
                || a.len() < extra + 3 + 3 * a[extra + 2] as usize
            {
                return None;
            }
            (
                a[extra],
                a[extra + 1],
                a[extra + 3..extra + 3 + 3 * a[extra + 2] as usize]
                    .chunks_exact(3)
                    .map(|v| (v[0], v[1], v[2]))
                    .collect(),
            )
        };
        let tail = if a.len() == extra {
            extra
        } else {
            extra + 3 + 3 * a[extra + 2] as usize
        };
        let mut profile = crate::athlete_ai_core::Profile::default();
        let mut scene = crate::athlete_ai_core::Scene::default();
        if a[0] == 3 {
            if a.len() != tail {
                return None;
            }
        } else {
            use crate::athlete_ai_core::{Actor, Objective, Profile, PROFILE_VERSION};
            if a.len() < tail + 12
                || a[tail] != PROFILE_VERSION
                || !(0..=64).contains(&a[tail + 11])
            {
                return None;
            }
            profile = Profile::new(a[tail + 2..tail + 11].try_into().ok()?);
            profile.known = a[tail + 1] != 0;
            let actors_end = tail + 12 + 14 * a[tail + 11] as usize;
            if a.len() <= actors_end
                || !(0..=8).contains(&a[actors_end])
                || a.len() != actors_end + 1 + 7 * a[actors_end] as usize
            {
                return None;
            }
            for u in a[tail + 12..actors_end].chunks_exact(14) {
                if u[0] < 0
                    || u[1] < 0
                    || u[5] <= 0
                    || u[7] <= 0
                    || scene.actors.iter().any(|v| v.id == u[0] as usize)
                {
                    return None;
                }
                scene.actors.push(Actor {
                    id: u[0] as usize,
                    team: u[1] as usize,
                    point: (u[2], u[3]),
                    hp: u[4],
                    max_hp: u[5],
                    attack: u[6],
                    speed: u[7],
                    vx: u[8],
                    vy: u[9],
                    visible: u[10] != 0,
                    controlled: u[11] != 0,
                    immune: u[12] != 0,
                    pressure: u[13].max(0),
                });
            }
            for o in a[actors_end + 1..].chunks_exact(7) {
                if o[0] < 0 || !(0..=1).contains(&o[1]) || o[5] <= 0 {
                    return None;
                }
                scene.objectives.push(Objective {
                    id: o[0] as usize,
                    kind: o[1] as u8,
                    point: (o[2], o[3]),
                    hp: o[4],
                    max_hp: o[5],
                    loss_per_second: o[6].max(0),
                });
            }
        }
        let mut all = vec![];
        for u in a[16..end].chunks_exact(15) {
            if u[0] < 0
                || u[1] < 0
                || u[4] < 0
                || u[5] <= 0
                || u[6] < 0
                || u[7] < 0
                || u[2] < 0
                || u[3] < 0
                || u[2] >= 960000
                || u[3] >= 960000
                || all.iter().any(|v: &Fighter| v.id == u[0] as usize)
            {
                return None;
            }
            all.push(Fighter {
                id: u[0] as usize,
                team: u[1] as usize,
                x: u[2],
                y: u[3],
                hp: u[4],
                max_hp: u[5],
                ad: u[6],
                ap: u[7],
                level: u[8],
                champion: u[9] != 0,
                visible: u[10] != 0,
                targetable: u[11] != 0,
                immune: u[12] & 1 != 0,
                cc_immune: u[12] & 2 != 0,
                immovable: u[12] & 4 != 0,
                cleansed: u[12] & 8 != 0,
                farmable: u[13] != 0,
                tower: u[14] != 0,
            });
        }
        if !all.iter().any(|u| u.id == a[2] as usize) {
            return None;
        }
        let target = usize::try_from(a[5]).ok()?;
        let action = match a[4] {
            0 => None,
            1 => Some(Action::Blue(target)),
            2 => Some(Action::MaxBlue(target)),
            3 => Some(Action::Red(target)),
            4 => Some(Action::MaxRed(target)),
            5 => Some(Action::Purple(target)),
            6 => Some(Action::Heal),
            7 => Some(Action::Blink(a[6], a[7])),
            8 => Some(Action::Domain),
            9 => Some(Action::QuickDomain),
            10 => Some(Action::OrbRed),
            11 => Some(Action::RedFeint(target, 1)),
            12 => Some(Action::MaxBlueCurve(target, 1)),
            13 => Some(Action::MaxBlueCurve(target, -1)),
            14 => Some(Action::RedFeint(target, -1)),
            _ => return None,
        };
        Some(Self {
            selected: target,
            aim: (a[6], a[7]),
            tick: a[1] as usize,
            owner: a[2] as usize,
            automatic: a[3] != 0,
            action,
            unlocked: [a[8] & 1 != 0, a[8] & 2 != 0, a[8] & 4 != 0],
            controlled: a[9] != 0,
            bound: a[10] != 0,
            basic: if a[11] > 0 {
                vec![(a[11] as u64, usize::try_from(a[12]).ok()?)]
            } else {
                vec![]
            },
            hit: a[13] != 0,
            shield: if a[14] < 0 { None } else { Some(a[14]) },
            all,
            walls: a[end + 1..extra]
                .chunks_exact(2)
                .map(|v| (v[0], v[1]))
                .collect(),
            haste,
            ult_haste,
            barriers,
            profile,
            scene,
        })
    }
    pub fn run(&self, c: &mut Core) -> String {
        c.profile = self.profile;
        c.scene = self.scene.clone();
        let wall = |x: i64, y: i64| {
            self.walls
                .contains(&(x.div_euclid(16000), y.div_euclid(16000)))
        };
        let me = self.all.iter().find(|u| u.id == self.owner).unwrap();
        let visual = crate::gojo_v2_presentation::Before::capture(c, self.hit);
        let input = Input {
            tick: self.tick,
            me,
            all: &self.all,
            unlocked: self.unlocked,
            controlled: self.controlled,
            bound: self.bound,
            automatic: self.automatic,
            haste: self.haste,
            ult_haste: self.ult_haste,
            action: self.action,
            basics: &self.basic,
            champion_damage: self.hit,
            shield: self.shield,
            wall: &wall,
            barrier: &|x, y| {
                self.barriers
                    .iter()
                    .any(|v| within((v.0, v.1), (x, y), v.2))
            },
            sight: &|x, y| within(me.pos(), (x, y), 32000) && !wall(x, y),
        };
        let commands = c.step(&input);
        let cmds = commands
            .iter()
            .map(|v| match v {
                Cmd::Damage {
                    target,
                    ad,
                    ap,
                    basic,
                    event,
                } => format!("[0,{target},{ad},{ap},{},{event}]", *basic as u8),
                Cmd::Move { target, x, y } => format!("[1,{target},{x},{y}]"),
                Cmd::Heal(n) => format!("[2,{n}]"),
                Cmd::Infinity { active, shield } => format!("[3,{},{shield}]", *active as u8),
                Cmd::Seal { target, ticks } => format!("[4,{target},{ticks}]"),
                Cmd::Slow { target, ticks, percent } => format!("[13,{target},{ticks},{percent}]"),
                Cmd::Guard {
                    target,
                    ticks,
                    slow,
                } => format!("[5,{target},{ticks},{}]", *slow as u8),
                Cmd::Fx { kind, x, y, ticks } => format!("[6,{kind},{x},{y},{ticks}]"),
                Cmd::Warning { x, y, r, ticks } => format!("[7,{x},{y},{r},{ticks}]"),
                Cmd::Lock(n) => format!("[8,{n}]"),
                Cmd::Hold { target, ticks } => format!("[11,{target},{ticks}]"),
                Cmd::SelfCost => "[12]".into(),
                Cmd::DomainEnd => "[9]".into(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let d = c.domain.as_ref().map_or("null".into(), |d| {
            format!(
                "[{},{},{},{},{},{},[{}],{}]",
                d.centre.0,
                d.centre.1,
                d.start,
                d.expand,
                d.end,
                d.guest.map_or(-1, |n| n as i64),
                d.trapped
                    .iter()
                    .map(|v| format!("[{},{},{}]", v.0, v.1, v.2))
                    .collect::<Vec<_>>()
                    .join(","),
                d.radius
            )
        });
        let presentation =
            crate::gojo_v2_presentation::json(&visual.cues(c, &commands, me, &self.all, self.tick));
        // Debug state is a quoted JSON string, not executable code.
        let mut result=format!("{{\"version\":2,\"eyes\":{},\"orb\":{},\"fusion\":{},\"tick\":{},\"ce\":{},\"strain\":{},\"cooldowns\":{:?},\"burnout\":{},\"amplifiedUntil\":{},\"flashes\":{},\"infinity\":{},\"shield\":{},\"domain\":{},\"lockUntil\":{},\"combo\":{},\"reason\":{:?},\"commands\":[{}],\"core\":{:?}}}",c.eyes,c.orb.as_ref().map_or("null".into(),|o|format!("[{},{},{}]",o.point.0,o.point.1,o.end)),c.fusion.as_ref().map_or("null".into(),|f|format!("[{},{},{}]",f.point.0,f.point.1,f.at)),self.tick,c.ce,c.strain,c.ready,c.burnout,c.amp_until,c.flashes,c.infinity,c.shield,d,c.lock_until,c.combo.as_ref().map_or("null".into(),|v|format!("[{},{},{}]",v.target,v.end,v.punches)),c.reason,cmds,format!("{c:?}"));
        result = result.replacen("\"version\":2", "\"version\":4", 1);
        result.pop();
        let availability = [
            Action::Blue(self.selected),
            Action::MaxBlue(self.selected),
            Action::Red(self.selected),
            Action::MaxRed(self.selected),
            Action::Purple(self.selected),
            Action::Heal,
            Action::Blink(self.aim.0, self.aim.1),
            Action::Domain,
            Action::QuickDomain,
            Action::OrbRed,
            Action::RedFeint(self.selected, 1),
            Action::MaxBlueCurve(self.selected, 1),
            Action::MaxBlueCurve(self.selected, -1),
            Action::RedFeint(self.selected, -1),
        ]
        .iter()
        .map(|a| format!("{:?}", c.rejection(&input, *a).unwrap_or("Ready")))
        .collect::<Vec<_>>()
        .join(",");
        let paths=c.shots.iter().map(|s|format!("{{\"id\":{},\"kind\":{},\"start\":{},\"end\":{},\"returning\":{},\"progress\":{},\"points\":{:?}}}",s.id,s.kind,s.born,s.at,s.returning,s.progress,s.path.iter().map(|p|[p.0,p.1]).collect::<Vec<_>>())).collect::<Vec<_>>().join(",");
        result.push_str(&format!(
            ",\"casts\":{:?},\"availability\":[{availability}],\"paths\":[{paths}]",
            c.casts
        ));
        result.push_str(&format!(",\"presentation\":{presentation}}}"));
        result.pop();
        let decision=c.last_engage.as_ref().map_or("null".into(),|p|format!("{{\"score\":{},\"caught\":{},\"guest\":{},\"arriving\":{},\"alliedLossTicks\":{},\"reason\":{:?}}}",p.score,p.caught,p.guest.map_or(-1,|n|n as i64),p.incoming,p.allied_loss_ticks,p.reason));
        result.push_str(&format!(
            ",\"athleteProfile\":{:?},\"profileKnown\":{},\"decision\":{decision}}}",
            self.profile.values, self.profile.known
        ));
        result
    }
}
