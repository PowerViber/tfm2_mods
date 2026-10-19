//! Read-only presentation projection. No combat mutation, event allocation or RNG.
use crate::gojo_v2_core::*;
use crate::gojo_v2_data::PREPARATIONS;
use crate::gojo_visual_data::{FRAMES, FRAME_TICKS, VERSION};

#[derive(Default)]
pub struct Before {
    shots: Vec<(usize, bool, usize, (i64, i64), bool)>,
    cast: Option<Cast>,
    contact: bool,
}
#[derive(Clone, Debug)]
pub struct Cue {
    pub slot: i64,
    pub tag: &'static str,
    pub phase: &'static str,
    pub start: usize,
    pub end: usize,
    pub x: i64,
    pub y: i64,
    pub left: bool,
    pub frame: usize,
    pub offset: [i64; 2],
}
impl Cue {
    pub fn window_name(&self, tick: usize) -> String {
        let total = if matches!(self.phase, "prepare" | "fusion") {
            self.end.saturating_sub(self.start).max(1)
        } else {
            FRAMES * FRAME_TICKS
        };
        let offset = if matches!(self.phase, "prepare" | "fusion") {
            tick.saturating_sub(self.start).min(total - 1)
        } else {
            tick.saturating_sub(self.start) % total
        };
        format!("{}_d{total}_t{offset}", self.name(false))
    }
    pub fn name(&self, sampled: bool) -> String {
        let mirrored = self.left
            && matches!(
                self.tag,
                "melee"
                    | "enhanced"
                    | "purple_prepare"
                    | "purple_flight"
                    | "purple_amplified"
                    | "red_flight"
                    | "max_red_flight"
                    | "step_prepare"
                    | "step_arrive"
                    | "gesture_blue"
                    | "gesture_red"
                    | "gesture_purple"
                    | "gesture_domain"
                    | "infinity_contact"
            );
        format!(
            "{}{}{}",
            self.tag,
            if mirrored { "_l" } else { "" },
            if sampled {
                self.frame.to_string()
            } else {
                String::new()
            }
        )
    }
}
impl Before {
    pub fn capture(c: &Core, contact: bool) -> Self {
        Self {
            contact,
            cast: c.pending.clone(),
            shots: c
                .shots
                .iter()
                .map(|s| (s.kind, s.max, s.born, s.start, s.dx < 0))
                .collect(),
        }
    }
    pub fn cues(
        &self,
        c: &Core,
        commands: &[Cmd],
        me: &Fighter,
        all: &[Fighter],
        tick: usize,
    ) -> Vec<Cue> {
        if me.hp <= 0 {
            return vec![];
        }
        let mut out = Vec::with_capacity(commands.len().min(48));
        let mut push = |slot, tag, phase, start: usize, end: usize, p: (i64, i64), left| {
            if out.len() < 48 {
                let elapsed = tick.saturating_sub(start);
                let frame = if matches!(phase, "prepare" | "fusion" | "impact") {
                    elapsed * FRAMES / end.saturating_sub(start).max(1)
                } else {
                    elapsed / FRAME_TICKS % FRAMES
                };
                out.push(Cue {
                    slot,
                    tag,
                    phase,
                    start,
                    end,
                    x: p.0,
                    y: p.1,
                    left,
                    frame: frame.min(FRAMES - 1),
                    offset: crate::gojo_visual_data::gesture_offset(tag, elapsed),
                });
            }
        };
        if let Some(p) = &c.pending {
            let k = p.action.index();
            let start = p.end - PREPARATIONS[k] as usize;
            let tag = match p.action {
                Action::Blue(_) => "blue_prepare",
                Action::MaxBlue(_) | Action::MaxBlueCurve(..) => "max_blue_prepare",
                Action::Red(_) | Action::RedFeint(..) => "red_prepare",
                Action::MaxRed(_) | Action::OrbRed => "max_red_prepare",
                Action::Purple(_) => "purple_prepare",
                Action::Heal => "rct_prepare",
                Action::Blink(..) => "step_prepare",
                _ => "",
            };
            push(-1, tag, "prepare", start, p.end, me.pos(), p.point.0 < me.x);
            if k <= 4 {
                push(
                    -2,
                    if k < 2 {
                        "gesture_blue"
                    } else if k < 4 {
                        "gesture_red"
                    } else {
                        "gesture_purple"
                    },
                    "prepare",
                    start,
                    p.end,
                    me.pos(),
                    p.point.0 < me.x,
                );
            }
        }
        if let Some(d) = &c.domain {
            if tick < d.start + d.expand {
                let left = all
                    .iter()
                    .filter(|u| {
                        u.team != me.team
                            && u.visible
                            && u.hp > 0
                            && within(me.pos(), u.pos(), 90000)
                    })
                    .min_by_key(|u| ((u.x - me.x).pow(2) + (u.y - me.y).pow(2), u.id))
                    .is_some_and(|u| u.x < me.x);
                push(
                    -2,
                    "gesture_domain",
                    "prepare",
                    d.start,
                    d.start + d.expand,
                    me.pos(),
                    left,
                );
            }
        }
        if let Some(f) = &c.fusion {
            push(
                -3,
                "fusion",
                "fusion",
                f.at.saturating_sub(48),
                f.at,
                f.point,
                false,
            );
        }
        let mut centres = vec![];
        for s in &c.spirals {
            if centres.contains(&s.centre) {
                continue;
            }
            centres.push(s.centre);
            let max = c
                .spirals
                .iter()
                .any(|v| v.centre == s.centre && v.turns == 24);
            let end = c
                .spirals
                .iter()
                .filter(|v| v.centre == s.centre)
                .map(|v| v.end.min(v.expire))
                .max()
                .unwrap_or(s.end);
            push(
                -5,
                if max {
                    "max_blue_impact"
                } else {
                    "blue_impact"
                },
                "spiral",
                s.begin,
                end,
                s.centre,
                false,
            );
        }
        if self.contact && c.infinity {
            let left = all
                .iter()
                .filter(|u| u.team != me.team && u.visible && u.hp > 0)
                .min_by_key(|u| ((u.x - me.x).abs() + (u.y - me.y).abs(), u.id))
                .is_some_and(|u| u.x < me.x);
            push(
                -4,
                "infinity_contact",
                "impact",
                tick,
                tick + 12,
                me.pos(),
                left,
            );
        }
        for (i, cmd) in commands.iter().enumerate() {
            match cmd {
                Cmd::Fx { kind, x, y, ticks } => {
                    let p = (*x, *y);
                    if matches!(*kind, 10 | 11 | 15) {
                        continue;
                    }
                    if *kind == 8 && centres.contains(&p) {
                        continue;
                    }
                    if commands.iter().any(|v| matches!(v, Cmd::Lock(_)))
                        && c.pending
                            .as_ref()
                            .is_some_and(|p| p.origin == (*x, *y) && p.end == tick + ticks)
                    {
                        continue;
                    }
                    let shot = c
                        .shots
                        .iter()
                        .map(|s| (s.kind, s.max, s.born, s.start, s.dx < 0))
                        .chain(self.shots.iter().copied())
                        .filter(|s| match *kind {
                            8 | 13 => s.0 < 2,
                            9 | 12 => s.0 == 2 || s.0 == 3,
                            7 => s.0 == 4,
                            _ => false,
                        })
                        .min_by_key(|s| (s.3 .0 - x).abs() + (s.3 .1 - y).abs());
                    let max = shot.is_some_and(|s| s.1);
                    let left = shot.map_or_else(
                        || self.cast.as_ref().is_some_and(|s| s.point.0 < s.origin.0),
                        |s| s.4,
                    );
                    let tag = match *kind {
                        1 => "enhanced",
                        2 => "black_flash",
                        3 => "rct",
                        4 => "step_arrive",
                        7 => {
                            if tick < c.amp_until {
                                "purple_amplified"
                            } else {
                                "purple_flight"
                            }
                        }
                        8 => {
                            if max {
                                "max_blue_impact"
                            } else {
                                "blue_impact"
                            }
                        }
                        9 => {
                            if max {
                                "max_red_impact"
                            } else {
                                "red_impact"
                            }
                        }
                        12 => {
                            if max {
                                "max_red_flight"
                            } else {
                                "red_flight"
                            }
                        }
                        13 => {
                            if max {
                                "max_blue_flight"
                            } else {
                                "blue_flight"
                            }
                        }
                        14 => "orb",
                        16 => "nuke",
                        _ => continue,
                    };
                    let phase = if matches!(*kind, 7 | 12 | 13) {
                        "flight"
                    } else if *kind == 14 {
                        "orb"
                    } else {
                        "impact"
                    };
                    let origin = if phase == "flight" {
                        shot.map_or(tick, |s| s.2)
                    } else if *kind == 14 {
                        c.orb.as_ref().map_or(tick, |o| {
                            o.end
                                .saturating_sub(crate::gojo_v2_data::ORB_TICKS as usize)
                        })
                    } else {
                        tick
                    };
                    push(i as i64, tag, phase, origin, tick + ticks, p, left);
                }
                Cmd::Damage {
                    target,
                    basic: true,
                    ..
                } if !commands
                    .iter()
                    .any(|v| matches!(v, Cmd::Fx { kind: 1 | 2, .. })) =>
                {
                    if let Some(u) = all.iter().find(|u| u.id == *target) {
                        push(
                            i as i64,
                            "melee",
                            "impact",
                            tick,
                            tick + 12,
                            u.pos(),
                            u.x < me.x,
                        );
                    }
                }
                Cmd::SelfCost => push(
                    i as i64,
                    "recoil",
                    "impact",
                    tick,
                    tick + 12,
                    me.pos(),
                    false,
                ),
                _ => {}
            }
        }
        out
    }
}
pub fn json(cues: &[Cue]) -> String {
    format!("{{\"version\":{VERSION},\"cues\":[{}]}}",cues.iter().map(|v|format!("{{\"slot\":{},\"tag\":{:?},\"phase\":{:?},\"start\":{},\"end\":{},\"x\":{},\"y\":{},\"left\":{},\"frame\":{},\"offset\":{:?}}}",v.slot,v.tag,v.phase,v.start,v.end,v.x,v.y,v.left,v.frame,v.offset)).collect::<Vec<_>>().join(","))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn me() -> Fighter {
        Fighter {
            id: 1,
            team: 0,
            x: 100000,
            y: 100000,
            hp: 900,
            max_hp: 900,
            ad: 70,
            ap: 81,
            level: 3,
            champion: true,
            visible: true,
            targetable: true,
            immune: false,
            cc_immune: false,
            farmable: false,
            tower: false,
            immovable: false,
            cleansed: false,
        }
    }
    #[test]
    fn preparation_variants_are_explicit_and_do_not_restart() {
        for (a, tag, ticks) in [
            (Action::Blue(2), "blue_prepare", 24),
            (Action::MaxBlue(2), "max_blue_prepare", 54),
            (Action::Red(2), "red_prepare", 24),
            (Action::MaxRed(2), "max_red_prepare", 60),
            (Action::Purple(2), "purple_prepare", 34),
        ] {
            let mut c = Core::default();
            c.pending = Some(Cast {
                action: a,
                end: 100 + ticks,
                point: (120000, 100000),
                origin: me().pos(),
            });
            for offset in [0, ticks / 2, ticks - 1] {
                let b = Before::capture(&c, false);
                let cues = b.cues(&c, &[], &me(), &[me()], 100 + offset);
                assert_eq!(cues[0].tag, tag);
                assert_eq!(cues[0].start, 100);
                assert_eq!(cues[0].frame, offset * 8 / ticks);
                assert!(cues[1].tag.starts_with("gesture_"));
            }
            c.pending = None;
            assert!(Before::default()
                .cues(&c, &[], &me(), &[me()], 101)
                .is_empty());
        }
    }
    #[test]
    fn flight_uses_launch_clock_and_impact_keeps_maximum_variant() {
        let mut c = Core::default();
        c.shots.push(Shot {
            born: 100,
            path: vec![],
            progress: 0,
            feint: 0,
            returning: false,
            id: 7,
            kind: 1,
            primary: Some(2),
            start: (100000, 100000),
            point: (80000, 100000),
            dx: -10000,
            dy: 0,
            remaining: 70000,
            at: 112,
            max: true,
            hits: vec![],
        });
        let b = Before::capture(&c, false);
        let cmd = Cmd::Fx {
            kind: 13,
            x: 95000,
            y: 100000,
            ticks: 2,
        };
        let a = b.cues(&c, &[cmd.clone()], &me(), &[me()], 103);
        let z = b.cues(&c, &[cmd], &me(), &[me()], 106);
        assert_eq!(a[0].tag, "max_blue_flight");
        assert_eq!(a[0].start, 100);
        assert_eq!(z[0].frame, 2);
        assert_eq!(a[0].frame, 1);
        c.shots.clear();
        let v = b.cues(
            &c,
            &[Cmd::Fx {
                kind: 8,
                x: 80000,
                y: 100000,
                ticks: 20,
            }],
            &me(),
            &[me()],
            112,
        );
        assert_eq!(v[0].tag, "max_blue_impact");
    }
    #[test]
    fn fusion_has_three_stages_and_owner_cost_has_its_own_accent() {
        let mut c = Core::default();
        c.fusion = Some(Fusion {
            point: (120000, 100000),
            at: 148,
        });
        for (tick, frame) in [(100, 0), (118, 3), (142, 7)] {
            let v = Before::default().cues(&c, &[], &me(), &[me()], tick);
            assert_eq!(v[0].tag, "fusion");
            assert_eq!(v[0].frame, frame);
        }
        c.fusion = None;
        let v = Before::default().cues(&c, &[Cmd::SelfCost], &me(), &[me()], 148);
        assert_eq!(v[0].tag, "recoil");
        assert_eq!(v[0].x, me().x);
    }
    #[test]
    fn confirmed_ordinary_impact_gets_one_mark_and_empowered_impacts_are_not_duplicated() {
        let c = Core::default();
        let mut enemy = me();
        enemy.id = 2;
        enemy.team = 1;
        enemy.x = 120000;
        let all = [me(), enemy];
        let d = Cmd::Damage {
            target: 2,
            ad: 70,
            ap: 28,
            basic: true,
            event: 42,
        };
        let a = Before::default().cues(&c, &[d.clone()], &me(), &all, 1);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].tag, "melee");
        let b = Before::default().cues(
            &c,
            &[
                d,
                Cmd::Fx {
                    kind: 2,
                    x: 120000,
                    y: 100000,
                    ticks: 18,
                },
            ],
            &me(),
            &all,
            1,
        );
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].tag, "black_flash");
    }
    #[test]
    fn dead_owners_emit_no_new_presentation_and_cues_are_bounded() {
        let c = Core::default();
        let mut dead = me();
        dead.hp = 0;
        let commands = vec![
            Cmd::Fx {
                kind: 2,
                x: 100000,
                y: 100000,
                ticks: 18
            };
            100
        ];
        assert!(Before::default()
            .cues(&c, &commands, &dead, &[dead.clone()], 3)
            .is_empty());
        assert_eq!(
            Before::default()
                .cues(&c, &commands, &me(), &[me()], 3)
                .len(),
            48
        );
    }
    #[test]
    fn metadata_is_optional_and_does_not_allocate_combat_events() {
        let core = super::Core::default();
        let before = format!("{core:?}");
        let _ = super::Before::capture(&core, false);
        assert_eq!(before, format!("{core:?}"));
        assert_eq!(super::json(&[]), "{\"version\":2,\"cues\":[]}");
    }
}
