//! Versioned practice transport, also used by the native parity-vector producer.
use crate::magnus_core::*;
#[derive(Clone, Debug)]
pub struct Frame {
    pub tick: usize,
    pub owner: Fighter,
    pub fighters: Vec<Fighter>,
    pub walls: Vec<[i64; 4]>,
    pub smokes: Vec<[i64; 3]>,
    pub captures: Vec<Capture>,
    pub hits: Vec<Hit>,
    pub hp: Vec<(usize, i64)>,
    pub controlled: bool,
    pub automatic: bool,
    pub rank: usize,
    pub action: Option<Action>,
}
pub fn fighter_values(f: &Fighter) -> [i64; 14] {
    [
        f.id as i64,
        f.team as i64,
        f.x,
        f.y,
        f.hp,
        f.max_hp,
        f.attack,
        f.ap,
        f.champion as i64,
        f.visible as i64,
        f.targetable as i64,
        (f.tank as i64) | ((f.ranged as i64) << 1) | ((f.friendly_chess as i64) << 2),
        f.mate_immune_until as i64,
        f.sight_source as i64,
    ]
}
fn fighter(a: &[i64]) -> Fighter {
    Fighter {
        id: a[0].max(0) as usize,
        team: a[1].max(0) as usize,
        x: a[2],
        y: a[3],
        hp: a[4],
        max_hp: a[5].max(1),
        attack: a[6],
        ap: a[7],
        champion: a[8] != 0,
        visible: a[9] != 0,
        targetable: a[10] != 0,
        tank: a[11] & 1 != 0,
        ranged: a[11] & 2 != 0,
        friendly_chess: a[11] & 4 != 0,
        mate_immune_until: a[12].max(0) as usize,
        sight_source: a[13] != 0,
    }
}
impl Frame {
    pub fn step(&self, core: &mut Core) -> Vec<Cmd> {
        core.rank = self.rank.min(7);
        let wall = |x, y| self.blocked(x, y);
        let sight = |a, b, c, d| self.sight_clear(a, b, c, d);
        core.step(&Input {
            tick: self.tick,
            owner: &self.owner,
            fighters: &self.fighters,
            wall: &wall,
            sight: &sight,
            captures: &self.captures,
            hits: &self.hits,
            piece_hp: &self.hp,
            controlled: self.controlled,
            automatic: self.automatic,
            action: self.action,
        })
    }
    pub fn new(tick: usize, owner: Fighter) -> Self {
        Self {
            tick,
            owner,
            fighters: vec![],
            walls: vec![],
            smokes: vec![],
            captures: vec![],
            hits: vec![],
            hp: vec![],
            controlled: false,
            automatic: false,
            rank: 3,
            action: None,
        }
    }
    pub fn values(&self) -> Vec<i64> {
        let a = self.action.map(action_key).unwrap_or((9, 0, 0, 0, 0));
        let mut v = vec![
            2,
            self.tick as i64,
            self.controlled as i64,
            self.automatic as i64,
            self.rank as i64,
            a.0 as i64,
            a.1 as i64,
            a.2 as i64,
            a.3 as i64,
            a.4 as i64,
            self.fighters.len() as i64,
            self.walls.len() as i64,
            self.smokes.len() as i64,
            self.captures.len() as i64,
            self.hp.len() as i64,
            self.hits.len() as i64,
        ];
        v.extend(fighter_values(&self.owner));
        for f in &self.fighters {
            v.extend(fighter_values(f));
        }
        for r in &self.walls {
            v.extend(r);
        }
        for r in &self.smokes {
            v.extend(r);
        }
        for c in &self.captures {
            v.extend([
                c.piece as i64,
                c.by.map_or(-1, |id| id as i64),
                c.combat as i64,
            ]);
        }
        for (id, hp) in &self.hp {
            v.extend([*id as i64, *hp]);
        }
        for h in &self.hits {
            v.extend([h.command as i64, h.piece as i64, h.target as i64, h.amount]);
        }
        v
    }
    pub fn decode(a: &[i64]) -> Option<Self> {
        if a.len() < 30 || a[0] != 2 {
            return None;
        }
        let counts: Vec<usize> = a[10..16]
            .iter()
            .map(|n| usize::try_from(*n))
            .collect::<Result<_, _>>()
            .ok()?;
        let widths = [14, 4, 3, 3, 2, 4];
        let expected = counts
            .iter()
            .zip(widths)
            .try_fold(30usize, |sum, (n, w)| sum.checked_add(n.checked_mul(w)?))?;
        if expected != a.len() {
            return None;
        }
        let owner = fighter(&a[16..30]);
        let kind = usize::try_from(a[6]).ok()?;
        let to = Cell::new(i32::try_from(a[7]).ok()?, i32::try_from(a[8]).ok()?);
        let heading = a[9].rem_euclid(4) as u8;
        let action = match a[5] {
            0 => Some(Action::Deploy { heading }),
            1 => Some(Action::Move { piece: kind, to }),
            2 => Some(Action::Castle { piece: kind }),
            3 => Some(Action::Dismiss),
            4 => Some(Action::Summon {
                kind: Kind::from_code(kind)?,
                to,
                direction: heading,
            }),
            5 => Some(Action::Promote {
                piece: kind,
                kind: Kind::from_code(usize::try_from(a[7]).ok()?)?,
            }),
            _ => None,
        };
        let mut cursor = 30;
        let mut take = |count: usize, width: usize| {
            let start = cursor;
            cursor += count * width;
            &a[start..cursor]
        };
        let fighters = take(counts[0], 14).chunks_exact(14).map(fighter).collect();
        let walls = take(counts[1], 4)
            .chunks_exact(4)
            .map(|r| r.try_into().unwrap())
            .collect();
        let smokes = take(counts[2], 3)
            .chunks_exact(3)
            .map(|r| r.try_into().unwrap())
            .collect();
        let captures = take(counts[3], 3)
            .chunks_exact(3)
            .map(|r| Capture {
                piece: r[0].max(0) as usize,
                by: usize::try_from(r[1]).ok(),
                combat: r[2] != 0,
            })
            .collect();
        let hp = take(counts[4], 2)
            .chunks_exact(2)
            .map(|r| (r[0].max(0) as usize, r[1]))
            .collect();
        let hits = take(counts[5], 4)
            .chunks_exact(4)
            .map(|r| Hit {
                command: r[0].max(0) as usize,
                piece: r[1].max(0) as usize,
                target: r[2].max(0) as usize,
                amount: r[3],
            })
            .collect();
        Some(Self {
            tick: a[1].max(0) as usize,
            controlled: a[2] != 0,
            automatic: a[3] != 0,
            rank: (a[4].max(0) as usize).min(7),
            action,
            owner,
            fighters,
            walls,
            smokes,
            captures,
            hits,
            hp,
        })
    }
    pub fn blocked(&self, x: i64, y: i64) -> bool {
        x < 0
            || y < 0
            || x >= 960_000
            || y >= 960_000
            || self
                .walls
                .iter()
                .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
    }
    pub fn sight_clear(&self, ax: i64, ay: i64, bx: i64, by: i64) -> bool {
        let n = (((bx - ax).abs().max((by - ay).abs()) / 2000) + 1).max(1);
        (0..=n).all(|k| {
            let (x, y) = (ax + (bx - ax) * k / n, ay + (by - ay) * k / n);
            !self.blocked(x, y)
                && !self
                    .smokes
                    .iter()
                    .any(|s| (x - s[0]).pow(2) + (y - s[1]).pow(2) <= s[2].pow(2))
        })
    }
    pub fn run(&self, core: &mut Core) -> (String, Vec<Cmd>) {
        core.rank = self.rank.min(7);
        let wall = |x, y| self.blocked(x, y);
        let sight = |a, b, c, d| self.sight_clear(a, b, c, d);
        let input = Input {
            tick: self.tick,
            owner: &self.owner,
            fighters: &self.fighters,
            wall: &wall,
            sight: &sight,
            captures: &self.captures,
            hits: &self.hits,
            piece_hp: &self.hp,
            controlled: self.controlled,
            automatic: self.automatic,
            action: self.action,
        };
        let commands = core.step(&input);
        (core.json(&input, &commands), commands)
    }
    pub fn state(&self, core: &Core) -> String {
        let wall = |x, y| self.blocked(x, y);
        let sight = |a, b, c, d| self.sight_clear(a, b, c, d);
        core.json(
            &Input {
                tick: self.tick,
                owner: &self.owner,
                fighters: &self.fighters,
                wall: &wall,
                sight: &sight,
                captures: &[],
                hits: &[],
                piece_hp: &[],
                controlled: self.controlled,
                automatic: self.automatic,
                action: None,
            },
            &[],
        )
    }
    pub fn inspection(
        &self,
        core: &mut Core,
        challenge: Option<usize>,
        piece: Option<usize>,
    ) -> String {
        let wall = |x, y| self.blocked(x, y);
        let sight = |a, b, c, d| self.sight_clear(a, b, c, d);
        let input = Input {
            tick: self.tick,
            owner: &self.owner,
            fighters: &self.fighters,
            wall: &wall,
            sight: &sight,
            captures: &[],
            hits: &[],
            piece_hp: &[],
            controlled: self.controlled,
            automatic: false,
            action: None,
        };
        if let Some(id) = challenge {
            return core.choose_challenge(id, &input).to_string();
        }
        let Some(id) = piece else {
            return "[]".into();
        };
        let Some(p) = core.pieces.get(&id) else {
            return "[]".into();
        };
        let mut cells = Vec::new();
        for x in p.cell.x - 7..=p.cell.x + 7 {
            for y in p.cell.y - 7..=p.cell.y + 7 {
                if core.threatens(id, Cell::new(x, y), &input)
                    && !self.blocked(i64::from(x) * TILE, i64::from(y) * TILE)
                {
                    cells.push(format!("[{x},{y}]"));
                }
            }
        }
        format!("[{}]", cells.join(","))
    }
    /// Apply emitted combat in the harness, then acknowledge actual HP/shield damage next tick.
    pub fn apply(&mut self, commands: &[Cmd]) {
        self.hits.clear();
        self.captures.clear();
        self.hp.clear();
        for c in commands {
            match c {
                Cmd::Damage {
                    command,
                    piece,
                    target,
                    amount,
                } => {
                    if let Some(f) = self.fighters.iter_mut().find(|f| f.id == *target) {
                        let loss = (*amount).min(f.hp.max(0));
                        f.hp -= loss;
                        self.hits.push(Hit {
                            command: *command,
                            piece: *piece,
                            target: *target,
                            amount: loss,
                        });
                    }
                }
                Cmd::Stun { target, ticks: _ } => {
                    if let Some(f) = self.fighters.iter_mut().find(|f| f.id == *target) {
                        f.mate_immune_until = self.tick + 720;
                    }
                }
                Cmd::Immune { target, ticks } => {
                    if let Some(f) = self.fighters.iter_mut().find(|f| f.id == *target) {
                        f.mate_immune_until = self.tick + ticks;
                    }
                }
                Cmd::OwnerPos { x, y } => {
                    self.owner.x = *x;
                    self.owner.y = *y;
                }
                _ => {}
            }
        }
        self.action = None;
    }
}
