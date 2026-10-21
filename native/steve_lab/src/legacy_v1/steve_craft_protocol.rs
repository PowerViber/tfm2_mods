//! SteveCraft Lab v1: bounded integer frames, owner-local sessions, complete state parity.
use crate::legacy_v1::steve_craft_core::*;
use crate::legacy_v1::steve_craft_geometry::*;
pub const VERSION: i64 = 1;
pub struct Frame {
    tick: usize,
    owner: usize,
    flags: i64,
    action: Option<Action>,
    home: Point,
    haste: i64,
    ult_haste: i64,
    units: Vec<Unit>,
    basics: Vec<(u64, usize)>,
    objects: Vec<(u32, i64)>,
    rects: Vec<[i64; 4]>,
    barriers:Vec<(Point,Point)>,
    acks: Vec<(u32, bool)>,
    receipts: Vec<(u64, i64)>,
    parry: bool,
}
pub fn action(a: &Action) -> String {
    match a {
        Action::Recipe {
            recipe,
            target,
            point,
        } => format!(
            "[1,{recipe},{},{},{}]",
            target.map_or(0, |v| v + 1),
            point.0,
            point.1
        ),
        Action::Prepare(r) => format!("[2,{r}]"),
        Action::Deploy(p) => format!("[3,0,0,{},{}]", p.0, p.1),
        Action::Detonate(id) => format!("[4,{id}]"),
        Action::Discard => "[5]".into(),
        Action::Stance => "[6]".into(),
        Action::Wall {
            centre,
            heading,
            bend,
        } => format!("[7,0,0,{},{},{heading},{bend}]", centre.0, centre.1),
        Action::Dismount(p) => format!("[8,0,0,{},{}]", p.0, p.1),
    }
}
fn points(p: &[Point]) -> String {
    format!(
        "[{}]",
        p.iter()
            .map(|p| format!("[{},{}]", p.0, p.1))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn command(c: &Cmd) -> String {
    match c {
        Cmd::Damage {
            event,
            target,
            ad,
            ap,
            utility,
        } => format!("[0,{event},{target},{ad},{ap},{}]", *utility as u8),
        Cmd::Push(id, p, t) => format!("[13,{id},{},{},{t}]", p.0, p.1),
        Cmd::Heal(id, n) => format!("[1,{id},{n}]"),
        Cmd::Shield(id, n, t) => format!("[2,{id},{n},{t}]"),
        Cmd::Move(id, p) => format!("[3,{id},{},{}]", p.0, p.1),
        Cmd::Slow(id, n, t) => format!("[4,{id},{n},{t}]"),
        Cmd::Speed(id, n, t) => format!("[5,{id},{n},{t}]"),
        Cmd::Spawn {
            object,
            recipe,
            point,
            hp,
        } => format!("[6,{object},{recipe},{},{},{hp}]", point.0, point.1),
        Cmd::Remove(id) => format!("[7,{id}]"),
        Cmd::Warning {
            point,
            radius,
            until,
            owner_hazard,
        } => format!(
            "[8,{},{},{radius},{until},{}]",
            point.0, point.1, *owner_hazard as u8
        ),
        Cmd::Lock { ticks, pose } => format!("[9,{ticks},{pose}]"),
        Cmd::Immune(t) => format!("[10,{t}]"),
        Cmd::Fx { kind, point } => format!("[11,{kind},{},{}]", point.0, point.1),
        Cmd::PathWarning { path, until } => format!("[12,{until},{}]", points(path)),
    }
}
impl Frame {
    pub fn decode(a: &[i64]) -> Option<Self> {
        if a.len() < 23
            || a.len() > 30000
            || a[0] != VERSION
            || a[1] < 0
            || a[1] > 100000000
            || a[2] < 0
            || a[2] > u32::MAX as i64
            || a[3] < 0
            || a[3] > 63 || a[22]<0 || a[22]>96
        {
            return None;
        }
        let counts = &a[15..21];
        if counts.iter().any(|&n| n < 0)
            || counts[0] > 1024
            || counts[1] > 64
            || counts[2] > 64
            || counts[3] > 256
            || counts[4] > 64
            || counts[5] > 256
        {
            return None;
        }
        if a.len()
            != 23 + a[22] as usize*4
                + counts[0] as usize * 20
                + counts[1] as usize * 2
                + counts[2] as usize * 2
                + counts[3] as usize * 4
                + counts[4] as usize * 2
                + counts[5] as usize * 2
        {
            return None;
        }
        let target = if a[6] == 0 {
            None
        } else if a[6] > 0 && a[6] <= u32::MAX as i64 {
            Some((a[6] - 1) as usize)
        } else {
            return None;
        };
        let p = (a[7], a[8]);
        let action = match a[4] {
            0 => None,
            1 if (0..15).contains(&a[5]) => Some(Action::Recipe {
                recipe: a[5] as u8,
                target,
                point: p,
            }),
            2 if a[5] == 6 || a[5] == 7 => Some(Action::Prepare(a[5] as u8)),
            3 => Some(Action::Deploy(p)),
            4 if (0..=u32::MAX as i64).contains(&a[5]) => Some(Action::Detonate(a[5] as u32)),
            5 => Some(Action::Discard),
            6 => Some(Action::Stance),
            7 if (0..16).contains(&a[9]) && (-4..=4).contains(&a[10]) => Some(Action::Wall {
                centre: p,
                heading: a[9] as u8,
                bend: a[10] as i8,
            }),
            8 => Some(Action::Dismount(p)),
            _ => return None,
        };
        if !(-2000000..=2000000).contains(&p.0) || !(-2000000..=2000000).contains(&p.1) {
            return None;
        }
        let mut n = 23;
        let mut units = vec![];
        for _ in 0..counts[0] {
            let v = &a[n..n + 20];
            if v[0] < 0
                || v[0] > u32::MAX as i64
                || v[1] < 0
                || v[1] > 16
                || !legal((v[2], v[3]))
                || v[5] <= 0
                || v[5] > 100000000
                || v[4] < 0
                || v[4] > 100000000
                || v[6] < 0
                || v[6] > 1000000
                || v[7] < 0
                || v[7] > 100000000
                || !(-1000000..=1000000).contains(&v[15])
                || !(-1000000..=1000000).contains(&v[16])
                || units.iter().any(|u: &Unit| u.id == v[0] as usize)
            {
                return None;
            }
            units.push(Unit {
                id: v[0] as usize,
                team: v[1] as usize,
                x: v[2],
                y: v[3],
                hp: v[4],
                max_hp: v[5],
                ad: v[6],
                shield: v[7],
                visible: v[8] != 0,
                targetable: v[9] != 0,
                champion: v[10] != 0,
                farmable: v[11] != 0,
                immune: v[12] != 0,
                cc_immune: v[13] != 0,
                immovable: v[14] != 0,
                vx: v[15],
                vy: v[16],
                tower: v[17] != 0,
                gadget: v[18] != 0,
                swing: v[19] != 0,
            });
            n += 20;
        }
        if !units.iter().any(|u| u.id == a[2] as usize) {
            return None;
        }
        let mut basics = vec![];
        for _ in 0..counts[1] {
            if a[n] < 0 || a[n + 1] < 0 {
                return None;
            }
            basics.push((a[n] as u64, a[n + 1] as usize));
            n += 2
        }
        let mut objects = vec![];
        for _ in 0..counts[2] {
            if a[n] < 0 || a[n] > u32::MAX as i64 || a[n + 1] < 0 {
                return None;
            }
            objects.push((a[n] as u32, a[n + 1]));
            n += 2
        }
        let mut rects = vec![];
        for _ in 0..counts[3] {
            let v = [a[n], a[n + 1], a[n + 2], a[n + 3]];
            if v.iter().any(|v| !(-2000000..=2000000).contains(v)) {
                return None;
            }
            rects.push(v);
            n += 4
        }
        let mut acks = vec![];
        for _ in 0..counts[4] {
            if a[n] < 0 || a[n] > u32::MAX as i64 {
                return None;
            }
            acks.push((a[n] as u32, a[n + 1] != 0));
            n += 2
        }
        let mut receipts = vec![];
        for _ in 0..counts[5] {
            if a[n] < 0 || a[n + 1] < 0 {
                return None;
            }
            receipts.push((a[n] as u64, a[n + 1]));
            n += 2
        }
        let mut barriers=vec![];for _ in 0..a[22]{let p=(a[n],a[n+1]);let q=(a[n+2],a[n+3]);if !legal(p)||!legal(q){return None}barriers.push((p,q));n+=4;}
        Some(Self {
            tick: a[1] as usize,
            owner: a[2] as usize,
            flags: a[3],
            action,
            home: (a[11], a[12]),
            haste: a[13].clamp(-90, 10000),
            ult_haste: a[14].clamp(-90, 10000),
            units,
            basics,
            objects,
            rects,
            acks,
            receipts,
            parry: a[21] != 0,barriers,
        })
    }
    pub fn run(self, c: &mut Core) -> String {
        if c.last_tick.is_some_and(|t| self.tick <= t) {
            return "{\"error\":\"Tick must increase\"}".into();
        }
        for (id, success) in self.acks {
            c.allocated(id, success)
        }
        for (event, actual) in self.receipts {
            c.receipt(event, actual, self.tick)
        }
        if self.parry {
            c.parried()
        }
        let wall = |x, y| {
            self.rects
                .iter()
                .any(|r| x >= r[0] && x <= r[2] && y >= r[1] && y <= r[3])
        };
        let me = self.units.iter().find(|u| u.id == self.owner).unwrap();
        let input = Input {
            tick: self.tick,
            me,
            units: &self.units,
            unlocked: [
                self.flags & 1 != 0,
                self.flags & 2 != 0,
                self.flags & 4 != 0,
            ],
            controlled: self.flags & 8 != 0,
            bound: self.flags & 16 != 0,
            automatic: self.flags & 32 != 0,
            action: self.action,
            basics: &self.basics,
            objects: &self.objects,
            wall: &wall,
            barriers: &self.barriers,
            home: self.home,
            haste: self.haste,
            ult_haste: self.ult_haste,
        };
        let out = c.step(&input);
        format!("{{\"version\":1,\"tick\":{},\"form\":{},\"charges\":{:?},\"cooldowns\":{:?},\"stanceAt\":{},\"ultAt\":{},\"pending\":{},\"prepared\":{},\"objects\":[{}],\"shots\":[{}],\"walls\":[{}],\"segments\":{},\"ride\":{},\"followup\":{},\"axeUntil\":{},\"reason\":{:?},\"selected\":{},\"plans\":[{}],\"commands\":[{}],\"core\":{:?}}}",self.tick,if c.form==Form::Steve{0}else{1},c.charges.iter().map(|v|match v{Charge::Ready(t)=>*t as i64,Charge::Reserved=>-1}).collect::<Vec<_>>(),c.cooldowns,c.stance_at,c.ult_at,c.pending.as_ref().map_or("null".into(),|p|format!("{{\"action\":{},\"begin\":{},\"end\":{},\"path\":{}}}",action(&p.action),p.begin,p.end,p.path.as_ref().map_or("null".into(),|v|points(v)))),c.prepared.as_ref().map_or("null".into(),|g|g.recipe.to_string()),c.objects.iter().map(|o|format!("{{\"id\":{},\"recipe\":{},\"x\":{},\"y\":{},\"hp\":{},\"confirmed\":{},\"end\":{},\"blast\":{}}}",o.id,o.recipe,o.point.0,o.point.1,o.hp,o.confirmed,o.end,o.blast.map_or("null".into(),|t|t.to_string()))).collect::<Vec<_>>().join(","),c.shots.iter().map(|s|format!("[{}, {}, {}, {}, {}]",s.recipe,s.point.0,s.point.1,s.dest.0,s.dest.1)).collect::<Vec<_>>().join(","),c.walls.iter().map(|w|format!("{{\"path\":{},\"begins\":{:?},\"finishes\":{:?},\"end\":{},\"ultimate\":{}}}",points(&w.path),w.begins,w.finishes,w.end,w.ultimate)).collect::<Vec<_>>().join(","),points(&c.segments(self.tick).iter().flat_map(|&(a,b)|[a,b]).collect::<Vec<_>>()),c.ride.as_ref().map_or("null".into(),|r|format!("{{\"path\":{},\"rider\":{},\"start\":{},\"end\":{}}}",points(&r.path),r.rider,r.start,r.end)),c.followup.map_or("null".into(),|(id,t)|format!("[{id},{t}]")),c.axe_until,c.reason,c.selected.as_ref().map_or("null".into(),action),c.preview.iter().map(|p|format!("{{\"actions\":[{}],\"score\":{}}}",p.actions.iter().map(action).collect::<Vec<_>>().join(","),p.score)).collect::<Vec<_>>().join(","),out.iter().map(command).collect::<Vec<_>>().join(","),format!("{:?}",c))
    }
}
