//! Version 1 bounded numeric input and complete combat/presentation output.
use crate::frieren_core::*;
pub struct Frame {
    tick: usize,
    owner: usize,
    automatic: bool,
    ult: bool,
    barrier: bool,
    action: Option<Action>,
    stark: Option<(i64, i64)>,
    units: Vec<Unit>,
    walls: Vec<(i64, i64)>,
}
impl Frame {
    pub fn decode(a: &[i64]) -> Option<Self> {
        if a.len() < 12 || a[0] != 1 || a[1] < 0 || a[2] < 0 || !(1..=512).contains(&a[11]) {
            return None;
        }
        let n = a[11] as usize;
        let end = 12 + 15 * n;
        if a.len() < end + 1 {
            return None;
        }
        let mut units = vec![];
        for u in a[12..end].chunks_exact(15) {
            if u[0] < 0 || u[1] < 0 || u[4] < 0 || u[5] <= 0 || u[6] < 0 {
                return None;
            }
            units.push(Unit {
                id: u[0] as usize,
                team: u[1] as usize,
                x: u[2],
                y: u[3],
                hp: u[4],
                max_hp: u[5],
                ap: u[6],
                attack: u[7],
                visible: u[8] != 0,
                targetable: u[9] != 0,
                champion: u[10] != 0,
                farmable: u[11] != 0,
                controlled: u[12] != 0,
                vx: u[13],
                vy: u[14],
            });
        }
        if !units.iter().any(|u| u.id == a[2] as usize) {
            return None;
        }
        if !(0..=1024).contains(&a[end]) || a.len() != end + 1 + 2 * a[end] as usize {
            return None;
        }
        let walls = a[end + 1..].chunks_exact(2).map(|v| (v[0], v[1])).collect();
        let action = match a[6] {
            0 => None,
            1 => Some(Action::Quick(a[7].try_into().ok()?)),
            2 => Some(Action::Charged(a[7].try_into().ok()?)),
            3 => Some(Action::Barrier(a[7].try_into().ok()?)),
            4 => Some(Action::Ultimate),
            _ => return None,
        };
        Some(Self {
            tick: a[1] as usize,
            owner: a[2] as usize,
            automatic: a[3] != 0,
            ult: a[4] != 0,
            barrier: a[5] != 0,
            action,
            stark: (a[8] != 0).then_some((a[9], a[10])),
            units,
            walls,
        })
    }
    pub fn run(&self, c: &mut Core) -> String {
        let m = self.units.iter().find(|u| u.id == self.owner).unwrap();
        let wall = |x: i64, y: i64| {
            self.walls
                .contains(&(x.div_euclid(16000), y.div_euclid(16000)))
        };
        let commands = c.step(&Input {
            tick: self.tick,
            owner: m,
            units: &self.units,
            automatic: self.automatic,
            ult_unlocked: self.ult,
            barrier_unlocked: self.barrier,
            action: self.action,
            stark: self.stark,
            wall: &wall,
        });
        let commands = commands
            .iter()
            .map(|a| match a {
                Cmd::Damage { target, amount } => format!("[0,{target},{amount}]"),
                Cmd::Shield {
                    target,
                    amount,
                    ticks,
                } => format!("[1,{target},{amount},{ticks}]"),
                Cmd::Empower { ticks } => format!("[2,{ticks}]"),
                Cmd::SpawnStark { hp, attack } => format!("[3,{hp},{attack}]"),
                Cmd::RemoveStark => "[4]".into(),
                Cmd::MoveStark { x, y } => format!("[5,{x},{y}]"),
                Cmd::Warning {
                    target,
                    ticks,
                    charged,
                } => format!("[6,{target},{ticks},{}]", usize::from(*charged)),
                Cmd::Shot {
                    x,
                    y,
                    dx,
                    dy,
                    charged,
                } => format!("[7,{x},{y},{dx},{dy},{}]", usize::from(*charged)),
                Cmd::BarrierArt { target } => format!("[8,{target}]"),
                Cmd::PartyArt => "[9]".into(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let pending = c.pending.as_ref().map_or_else(
            || "null".into(),
            |p| format!("[{}, {}, {}]", p.target, p.end, usize::from(p.charged)),
        );
        let rays = c
            .rays
            .iter()
            .map(|r| {
                format!(
                    "[{}, {}, {}, {}, {}, {}, {}, {:?}]",
                    r.id, r.x, r.y, r.dx, r.dy, r.remaining, r.damage, r.hits
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let mut output=format!("{{\"version\":1,\"tick\":{},\"mana\":{},\"cooldowns\":[{},{},{}],\"ultimateUntil\":{},\"starkUntil\":{},\"guard\":{},\"selected\":{},\"pending\":{},\"rays\":[{}],\"reason\":\"{}\",\"casts\":{:?},\"interrupted\":{},\"requestedDamage\":{},\"nextFern\":{},\"nextStark\":{},\"commands\":[{}]}}",self.tick,c.mana,c.skill_at,c.barrier_at,c.ult_at,c.ult_until,c.stark_until,c.guard.map_or_else(||"null".into(),|n|n.to_string()),c.selected.map_or_else(||"null".into(),|n|n.to_string()),pending,rays,c.reason,c.casts,c.interrupted,c.requested_damage,c.next_fern,c.next_stark,commands);
        output.pop();
        output.push_str(",\"core\":");
        output.push_str(&format!("{:?}", format!("{c:?}")));
        output.push('}');
        output
    }
}
