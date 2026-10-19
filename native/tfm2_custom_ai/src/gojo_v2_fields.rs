//! Bounded field effects. No extra damage from orbiting or orb renewals.
use crate::gojo_v2_core::*;
use crate::gojo_v2_data::*;
impl Core {
    pub(super) fn spiral(
        &mut self,
        input: &Input,
        u: &Fighter,
        centre: (i64, i64),
        max: bool,
        out: &mut Vec<Cmd>,
    ) {
        if u.immune || u.cc_immune || u.immovable || u.cleansed {
            return;
        }
        let ticks = SPIRAL_TICKS[usize::from(max)] as usize;
        self.motions.retain(|m| m.target != u.id);
        self.spirals.retain(|m| m.target != u.id);
        let vector = if u.pos() == centre {
            (4000, 0)
        } else {
            (u.x - centre.0, u.y - centre.1)
        };
        self.spirals.push(Spiral {
            target: u.id,
            centre,
            vector,
            begin: input.tick,
            end: input.tick + ticks,
            expire: input.tick + ticks + 1,
            turns: if max { 24 } else { 18 },
        });
        out.push(Cmd::Hold {
            target: u.id,
            ticks,
        });
    }
    pub(super) fn spiral_tick(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        const COS: [i64; 24] = [
            10000, 9659, 8660, 7071, 5000, 2588, 0, -2588, -5000, -7071, -8660, -9659, -10000,
            -9659, -8660, -7071, -5000, -2588, 0, 2588, 5000, 7071, 8660, 9659,
        ];
        let mut keep = vec![];
        for m in std::mem::take(&mut self.spirals) {
            if input.tick >= m.expire {
                continue;
            }
            let Some(u) = input.all.iter().find(|u| {
                u.id == m.target
                    && u.hp > 0
                    && !u.immune
                    && !u.cc_immune
                    && !u.immovable
                    && !u.cleansed
            }) else {
                continue;
            };
            let duration = (m.end - m.begin).max(1);
            let elapsed = input.tick.saturating_sub(m.begin).min(duration);
            let phase = elapsed * m.turns * 1000 / duration;
            let n = (phase / 1000) % 24;
            let f = (phase % 1000) as i64;
            let c = (COS[n] * (1000 - f) + COS[(n + 1) % 24] * f) / 1000;
            let si = (COS[(n + 18) % 24] * (1000 - f) + COS[(n + 19) % 24] * f) / 1000;
            let v = crate::unified_theory_math::unit(m.vector.0, m.vector.1);
            let radius = (distance((0, 0), m.vector).isqrt() as i64).max(4000);
            let r = radius + (4000 - radius) * elapsed as i64 / duration as i64;
            let p = (
                m.centre.0 + (v.0 * c - v.1 * si) * r / 100000000,
                m.centre.1 + (v.0 * si + v.1 * c) * r / 100000000,
            );
            if self.route(input, u.pos(), p) {
                out.push(Cmd::Move {
                    target: u.id,
                    x: p.0,
                    y: p.1,
                });
                if input.tick < m.end {
                    keep.push(m);
                }
            }
        }
        self.spirals = keep;
    }
    pub(super) fn field_tick(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        if let Some(mut o) = self.orb.take() {
            if input.tick < o.end {
                for u in input.all.iter().filter(|u| {
                    u.hp > 0
                        && u.team != input.me.team
                        && u.targetable
                        && !u.tower
                        && within(o.point, u.pos(), 36000)
                }) {
                    if !o.affected.contains(&u.id) {
                        o.affected.push(u.id);
                        self.spiral(input, u, o.point, true, out);
                        if let Some(s) = self.spirals.iter_mut().find(|s| s.target == u.id) {
                            s.expire = s.expire.min(o.end);
                        }
                        if let Some(Cmd::Hold { target, ticks }) = out.last_mut() {
                            if *target == u.id {
                                *ticks = (*ticks).min(o.end - input.tick);
                            }
                        }
                    }
                }
                if input.tick % 6 == 0 {
                    Self::fx(out, 14, o.point, 8.min(o.end - input.tick));
                }
                self.orb = Some(o);
            }
        }
        if self.fusion.as_ref().is_some_and(|f| input.tick >= f.at) {
            let f = self.fusion.take().unwrap();
            for u in input.all.iter().filter(|u| {
                u.hp > 0
                    && u.team != input.me.team
                    && u.targetable
                    && !u.tower
                    && within(f.point, u.pos(), NUKE_RADIUS)
            }) {
                self.damage(
                    input,
                    u,
                    0,
                    NUKE_DAMAGE[0] + input.me.ap * NUKE_DAMAGE[1] / 100,
                    false,
                    out,
                );
            }
            out.push(Cmd::SelfCost);
            Self::fx(out, 16, f.point, 36);
            self.strain = (self.strain + NUKE_STRAIN).min(STRAIN_MAX);
            self.last_technique = input.tick;
        }
    }
}
