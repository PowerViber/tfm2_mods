// Deterministic map-wide Magnus rules. Shared verbatim by native and WASM.
use crate::magnus_data::{Pattern, GAMES};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
pub const TILE: i64 = 24_000;
pub const RESET: usize = 1440;
pub const REWARD_CD: usize = 1200;
pub const MATE_CD: usize = 1200;
pub const MATE_WARN: usize = 24;
pub const CASTLE_CD: usize = 720;
pub const RANKS: [&str; 8] = [
    "Beginner",
    "Club Player",
    "Tactician",
    "Expert",
    "Master",
    "Grandmaster",
    "Super GM",
    "World Top 10",
];
pub const PIECE_KINDS: [Kind; 7] = [
    Kind::Rook,
    Kind::Rook,
    Kind::Bishop,
    Kind::Bishop,
    Kind::Knight,
    Kind::Knight,
    Kind::Queen,
];
const DIRS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Rook,
    Bishop,
    Knight,
    Queen,
    Pawn,
}
impl Kind {
    pub fn code(self) -> usize {
        self as usize
    }
    pub fn from_code(n: usize) -> Option<Self> {
        [
            Self::Rook,
            Self::Bishop,
            Self::Knight,
            Self::Queen,
            Self::Pawn,
        ]
        .get(n)
        .copied()
    }
    pub fn name(self) -> &'static str {
        ["rook", "bishop", "knight", "queen", "pawn"][self.code()]
    }
    pub fn hp_pct(self) -> i64 {
        [30, 20, 25, 35, 15][self.code()]
    }
    pub fn damage(self, ap: i64) -> i64 {
        [35, 30, 45, 65, 20][self.code()] + ap.max(0) * [35, 40, 35, 55, 25][self.code()] / 100
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
}
impl Role {
    pub fn code(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}
impl Cell {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn world(self) -> (i64, i64) {
        (i64::from(self.x) * TILE, i64::from(self.y) * TILE)
    }
    pub fn at(x: i64, y: i64) -> Self {
        Self::new(
            (x + TILE / 2).div_euclid(TILE) as i32,
            (y + TILE / 2).div_euclid(TILE) as i32,
        )
    }
    pub fn distance(self, b: Self) -> i32 {
        (self.x - b.x).abs().max((self.y - b.y).abs())
    }
    pub fn valid(self) -> bool {
        (0..8).contains(&self.x) && (0..8).contains(&self.y)
    }
    pub fn bit(self) -> u64 {
        if self.valid() {
            1u64 << (self.y * 8 + self.x)
        } else {
            0
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub x: i64,
    pub y: i64,
    pub heading: u8,
    pub serial: usize,
}
impl Board {
    pub fn world(self, c: Cell) -> (i64, i64) {
        let (x, y) = ((i64::from(c.x) - 4) * TILE, i64::from(c.y) * TILE);
        let (x, y) = match self.heading % 4 {
            0 => (x, y),
            1 => (y, -x),
            2 => (-x, -y),
            _ => (-y, x),
        };
        (self.x + x, self.y + y)
    }
    pub fn cell(self, x: i64, y: i64) -> Cell {
        let (x, y) = (x - self.x, y - self.y);
        let (x, y) = match self.heading % 4 {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            _ => (y, -x),
        };
        Cell::new(
            (x + TILE / 2).div_euclid(TILE) as i32 + 4,
            (y + TILE / 2).div_euclid(TILE) as i32,
        )
    }
    pub fn inside(self, x: i64, y: i64) -> bool {
        self.cell(x, y).valid()
    }
    pub fn grid(self, c: Cell) -> Cell {
        let (x, y) = self.world(c);
        Cell::at(x, y)
    }
}
#[derive(Clone, Debug)]
pub struct Fighter {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub ap: i64,
    pub champion: bool,
    pub visible: bool,
    pub targetable: bool,
    pub tank: bool,
    pub ranged: bool,
    pub friendly_chess: bool,
    pub sight_source: bool,
    pub mate_immune_until: usize,
}
impl Fighter {
    pub fn cell(&self) -> Cell {
        Cell::at(self.x, self.y)
    }
}
#[derive(Clone, Debug)]
pub struct Piece {
    pub id: usize,
    pub kind: Kind,
    pub cell: Cell,
    pub hp: i64,
    pub max_hp: i64,
    pub slot: Option<usize>,
    pub generation: usize,
    pub direction: u8,
    pub moved: bool,
    pub ranks: u8,
    pub qualified: bool,
    pub exposed: bool,
    pub yielding: bool,
    pub promotion: Kind,
    pub intercept: Option<(usize, Cell, usize)>,
}
impl Piece {
    pub fn new(
        id: usize,
        kind: Kind,
        cell: Cell,
        max_hp: i64,
        slot: Option<usize>,
        direction: u8,
    ) -> Self {
        Self {
            id,
            kind,
            cell,
            hp: max_hp,
            max_hp,
            slot,
            generation: 0,
            direction,
            moved: false,
            ranks: 0,
            qualified: false,
            exposed: false,
            yielding: false,
            promotion: Kind::Queen,
            intercept: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub id: usize,
    pub role: Role,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Deploy { heading: u8 },
    Move { piece: usize, to: Cell },
    Castle { piece: usize },
    Dismiss,
    Summon { kind: Kind, to: Cell, direction: u8 },
    Promote { piece: usize, kind: Kind },
}
#[derive(Clone, Copy, Debug)]
pub struct Capture {
    pub piece: usize,
    pub by: Option<usize>,
    pub combat: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub command: usize,
    pub piece: usize,
    pub target: usize,
    pub amount: i64,
}
#[derive(Clone, Copy)]
pub struct Input<'a> {
    pub tick: usize,
    pub owner: &'a Fighter,
    pub fighters: &'a [Fighter],
    pub wall: &'a dyn Fn(i64, i64) -> bool,
    pub sight: &'a dyn Fn(i64, i64, i64, i64) -> bool,
    pub captures: &'a [Capture],
    pub hits: &'a [Hit],
    pub piece_hp: &'a [(usize, i64)],
    pub controlled: bool,
    pub automatic: bool,
    pub action: Option<Action>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    Spawn {
        piece: usize,
        x: i64,
        y: i64,
        hp: i64,
    },
    Despawn {
        piece: usize,
    },
    Move {
        piece: usize,
        x: i64,
        y: i64,
    },
    Damage {
        command: usize,
        piece: usize,
        target: usize,
        amount: i64,
    },
    Stun {
        target: usize,
        ticks: usize,
    },
    Immune {
        target: usize,
        ticks: usize,
    },
    Shield {
        target: Option<usize>,
        amount: i64,
        ticks: usize,
    },
    Buff {
        target: Option<usize>,
        kind: usize,
        ticks: usize,
    },
    OwnerPos {
        x: i64,
        y: i64,
    },
    Fx {
        tag: String,
        x: i64,
        y: i64,
        ticks: usize,
    },
    Label {
        code: usize,
    },
    Failure {
        reason: usize,
    },
    Appearance {
        piece: usize,
    },
}
#[derive(Clone, Debug)]
pub struct Pending {
    pub action: Action,
    pub start: usize,
    pub commit: usize,
    pub end: usize,
    pub from: Cell,
    pub hit: BTreeSet<usize>,
    pub committed: bool,
    pub yield_move: bool,
    pub multiplier: i64,
    pub before: BTreeMap<usize, Vec<usize>>,
}
#[derive(Clone, Debug)]
pub struct Challenge {
    pub id: usize,
    pub progress: usize,
    pub started: Option<usize>,
    pub done: bool,
    pub failed: bool,
    pub failure: usize,
    pub bindings: [Option<usize>; 5],
    pub target: Option<usize>,
    pub queen_target: Option<usize>,
    pub king_cell: Option<Cell>,
    pub pawn_start: u8,
    pub board_serial: Option<usize>,
    pub owner_last: Cell,
    pub owner_steps: usize,
    pub last_event: Option<(usize, usize, usize, Option<usize>)>,
    pub finished: Option<usize>,
}
impl Challenge {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            progress: 0,
            started: None,
            done: false,
            failed: false,
            failure: 0,
            bindings: [None; 5],
            target: None,
            queen_target: None,
            king_cell: None,
            pawn_start: 0,
            board_serial: None,
            owner_last: Cell::default(),
            owner_steps: 0,
            last_event: None,
            finished: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub commands: [usize; 5],
    pub captures: usize,
    pub mates: usize,
    pub rewards: usize,
    pub resets: usize,
    pub damage: i64,
    pub casts: usize,
    pub cancelled: usize,
    pub summons: usize,
    pub promotions: usize,
    pub field_mates: usize,
}
#[derive(Clone, Debug)]
pub struct Reward {
    pub id: usize,
    pub until: usize,
    pub charges: usize,
    pub piece: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Core {
    pub owner: usize,
    pub rank: usize,
    pub board: Option<Board>,
    pub pieces: BTreeMap<usize, Piece>,
    pub slots: [Option<usize>; 7],
    pub next_id: usize,
    pub projections: Vec<Projection>,
    pub pending: Option<Pending>,
    pub challenge: Option<Challenge>,
    pub previous_challenge: Option<usize>,
    pub reset_at: usize,
    pub pawn_at: usize,
    pub develop_at: usize,
    pub castle_at: usize,
    pub mate_at: usize,
    pub field_mate_at: usize,
    pub reward_at: usize,
    pub repeat_at: [usize; 12],
    pub reward: Option<Reward>,
    pub outside_since: Option<usize>,
    pub mate_warning: Option<usize>,
    pub mate_target: Option<usize>,
    pub mate_cell: Option<Cell>,
    pub mate_board: bool,
    pub field_target: Option<usize>,
    pub has_move: bool,
    pub stats: Stats,
    pub last_label: usize,
    pub label_until: usize,
    pub last_tick: Option<usize>,
    pub glow: BTreeSet<usize>,
    pub glow_until: usize,
    pub last_failure: usize,
    rng: u64,
    serial: usize,
    last_fx: usize,
    token_tick: usize,
    tokens: usize,
    fx_tick: usize,
    fx_count: usize,
    positions: BTreeMap<usize, (Cell, usize)>,
    owner_position: Option<(i64, i64)>,
    occupancy: BTreeMap<Cell, usize>,
    immune: BTreeMap<usize, usize>,
    pub planner_cursor: usize,
    pub last_plan_candidates: usize,
    awaiting: BTreeMap<(usize, usize, usize), (usize, bool)>,
    confirmed: BTreeMap<(usize, usize, usize), bool>,
}
impl Core {
    pub fn new(seed: u64, owner: usize, rank: usize) -> Self {
        Self {
            owner,
            rank: rank.min(7),
            board: None,
            pieces: BTreeMap::new(),
            slots: [None; 7],
            next_id: 0,
            projections: vec![],
            pending: None,
            challenge: None,
            previous_challenge: None,
            reset_at: 0,
            pawn_at: 0,
            develop_at: 0,
            castle_at: 0,
            mate_at: 0,
            field_mate_at: 0,
            reward_at: 0,
            repeat_at: [0; 12],
            reward: None,
            outside_since: None,
            mate_warning: None,
            mate_target: None,
            mate_cell: None,
            mate_board: false,
            field_target: None,
            has_move: false,
            stats: Stats::default(),
            last_label: 0,
            label_until: 0,
            last_tick: None,
            glow: BTreeSet::new(),
            glow_until: 0,
            last_failure: 0,
            rng: (seed ^ (owner as u64).wrapping_mul(0x9e3779b97f4a7c15)).max(1),
            serial: 0,
            last_fx: 0,
            token_tick: 0,
            tokens: 12,
            fx_tick: 0,
            fx_count: 0,
            positions: BTreeMap::new(),
            owner_position: None,
            occupancy: BTreeMap::new(),
            immune: BTreeMap::new(),
            planner_cursor: 0,
            last_plan_candidates: 0,
            awaiting: BTreeMap::new(),
            confirmed: BTreeMap::new(),
        }
    }
    pub fn roll(&mut self, n: usize) -> usize {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        (x % n.max(1) as u64) as usize
    }
    pub fn fx(
        &mut self,
        tick: usize,
        tag: &str,
        x: i64,
        y: i64,
        ticks: usize,
        ambient: bool,
        out: &mut Vec<Cmd>,
    ) {
        let refill = tick.saturating_sub(self.token_tick) / 2;
        self.tokens = (self.tokens + refill).min(12);
        self.token_tick += refill * 2;
        if self.fx_tick != tick {
            self.fx_tick = tick;
            self.fx_count = 0;
        }
        if self.tokens == 0 || self.fx_count >= 6 || (ambient && self.tokens <= 4) {
            return;
        }
        self.tokens -= 1;
        self.fx_count += 1;
        out.push(Cmd::Fx {
            tag: tag.into(),
            x,
            y,
            ticks,
        });
    }
    fn label(&mut self, code: usize, input: &Input, out: &mut Vec<Cmd>) {
        self.last_label = code;
        self.label_until = input.tick + 90;
        out.push(Cmd::Label { code });
        self.fx(
            input.tick,
            &format!("label{}", code.min(11)),
            input.owner.x,
            input.owner.y,
            90,
            false,
            out,
        );
    }
    pub fn active_count(&self) -> usize {
        self.pieces.len()
    }
    pub fn army_denominator(&self) -> i64 {
        100 + 8 * self.active_count().saturating_sub(7) as i64
    }
    pub fn duration(&self, kind: Kind, tick: usize) -> usize {
        let base = if kind == Kind::Queen { 45 } else { 30 };
        if self.reward.as_ref().is_some_and(|r| {
            tick < r.until && r.charges > 0 && (r.id == 2 || (r.id == 11 && kind == Kind::Pawn))
        }) {
            base * 4 / 5
        } else {
            base
        }
    }
    pub fn reindex(&mut self) {
        self.occupancy = self.pieces.values().map(|p| (p.cell, p.id)).collect();
    }
    fn visible<'a>(&self, input: &'a Input, id: usize) -> Option<&'a Fighter> {
        input
            .fighters
            .iter()
            .find(|f| f.id == id && f.hp > 0 && f.visible && f.targetable)
    }
    pub fn projected(&self, role: Role) -> Option<usize> {
        self.projections
            .iter()
            .find(|p| p.role == role)
            .map(|p| p.id)
    }
    fn project(&mut self, input: &Input, initial: bool) {
        let foes: Vec<_> = input
            .fighters
            .iter()
            .filter(|f| {
                f.team != input.owner.team
                    && f.hp > 0
                    && f.champion
                    && f.visible
                    && self.board.is_none_or(|b| b.inside(f.x, f.y))
            })
            .collect();
        if initial {
            self.projections.clear();
            let engaged = self.field_target.filter(|_| self.board.is_none());
            if let Some(k) = foes.iter().min_by_key(|f| {
                (
                    Some(f.id) != engaged,
                    (f.x - input.owner.x).pow(2) + (f.y - input.owner.y).pow(2),
                    f.id,
                )
            }) {
                self.projections.push(Projection {
                    id: k.id,
                    role: Role::King,
                });
            }
            if let Some(q) = foes
                .iter()
                .filter(|f| Some(f.id) != self.projected(Role::King))
                .max_by_key(|f| (f.attack + f.ap, std::cmp::Reverse(f.id)))
            {
                self.projections.push(Projection {
                    id: q.id,
                    role: Role::Queen,
                });
            }
        }
        for f in foes {
            if !self.projections.iter().any(|p| p.id == f.id) {
                self.projections.push(Projection {
                    id: f.id,
                    role: if f.tank {
                        Role::Rook
                    } else if f.ranged {
                        Role::Bishop
                    } else {
                        Role::Knight
                    },
                });
            }
        }
    }
    fn occupied(&self, c: Cell, except: usize, input: &Input) -> bool {
        c == input.owner.cell()
            || self.occupancy.get(&c).is_some_and(|id| *id != except)
            || input.fighters.iter().any(|f| {
                f.team == input.owner.team && f.friendly_chess && f.hp > 0 && f.cell() == c
            })
    }
    fn clear_segment(&self, a: Cell, b: Cell, input: &Input) -> bool {
        let (ax, ay) = a.world();
        let (bx, by) = b.world();
        let n = (a.distance(b) as i64 * 12).max(1);
        (0..=n).all(|k| !(input.wall)(ax + (bx - ax) * k / n, ay + (by - ay) * k / n))
    }
    pub fn legal(&self, id: usize, to: Cell, input: &Input) -> bool {
        let Some(p) = self.pieces.get(&id) else {
            return false;
        };
        if to == p.cell || self.occupied(to, id, input) || !self.clear_segment(to, to, input) {
            return false;
        }
        let (dx, dy) = (to.x - p.cell.x, to.y - p.cell.y);
        if p.kind == Kind::Knight {
            return dx.abs() * dy.abs() == 2 && dx.abs() + dy.abs() == 3;
        }
        if p.kind == Kind::Pawn {
            let (fx, fy) = forward(p.direction);
            let advance = dx * fx + dy * fy;
            let side = dx * fy - dy * fx;
            if side == 0 && (advance == 1 || advance == 2 && !p.moved) {
                let mid = Cell::new(p.cell.x + fx, p.cell.y + fy);
                return !self.occupied(mid, id, input)
                    && self.clear_segment(p.cell, to, input)
                    && !input.fighters.iter().any(|f| {
                        f.hp > 0
                            && f.visible
                            && f.team != input.owner.team
                            && (f.cell() == mid || f.cell() == to)
                    });
            }
            if advance == 1 && side.abs() == 1 {
                let enemy = input.fighters.iter().any(|f| {
                    f.team != input.owner.team
                        && f.hp > 0
                        && f.visible
                        && f.targetable
                        && f.cell() == to
                });
                let ep = p.intercept.is_some_and(|(target, c, until)| {
                    input.tick <= until
                        && c == to
                        && self.visible(input, target).is_some_and(|f| f.cell() == c)
                });
                return (enemy || ep) && self.clear_segment(p.cell, to, input);
            }
            return false;
        }
        if p.cell.distance(to) > 7 {
            return false;
        }
        let straight = dx == 0 || dy == 0;
        let diagonal = dx.abs() == dy.abs();
        if !match p.kind {
            Kind::Rook => straight,
            Kind::Bishop => diagonal,
            Kind::Queen => straight || diagonal,
            _ => false,
        } {
            return false;
        }
        let (sx, sy) = (dx.signum(), dy.signum());
        for n in 1..=p.cell.distance(to) {
            let c = Cell::new(p.cell.x + sx * n, p.cell.y + sy * n);
            if self.occupied(c, id, input) {
                return false;
            }
        }
        self.clear_segment(p.cell, to, input)
    }
    pub fn threatens(&self, id: usize, to: Cell, input: &Input) -> bool {
        let Some(p) = self.pieces.get(&id) else {
            return false;
        };
        if p.yielding
            || p.cell == input.owner.cell()
            || self.pending.as_ref().is_some_and(|m| {
                m.committed && matches!(m.action,Action::Move{piece,..}if piece==id)
            })
        {
            return false;
        }
        let (dx, dy) = (to.x - p.cell.x, to.y - p.cell.y);
        if p.kind == Kind::Pawn {
            let (fx, fy) = forward(p.direction);
            return dx * fx + dy * fy == 1
                && (dx * fy - dy * fx).abs() == 1
                && self.clear_segment(p.cell, to, input);
        }
        if p.kind == Kind::Knight {
            return dx.abs() * dy.abs() == 2
                && dx.abs() + dy.abs() == 3
                && self.clear_segment(to, to, input);
        }
        if p.cell == to || p.cell.distance(to) > 7 {
            return false;
        }
        let straight = dx == 0 || dy == 0;
        let diagonal = dx.abs() == dy.abs();
        if !match p.kind {
            Kind::Rook => straight,
            Kind::Bishop => diagonal,
            Kind::Queen => straight || diagonal,
            _ => false,
        } {
            return false;
        }
        for n in 1..p.cell.distance(to) {
            if self.occupied(
                Cell::new(p.cell.x + dx.signum() * n, p.cell.y + dy.signum() * n),
                id,
                input,
            ) {
                return false;
            }
        }
        self.clear_segment(p.cell, to, input)
    }
    pub fn coverage(&self, id: usize, input: &Input) -> u64 {
        self.board.map_or(0, |b| {
            (0..64).fold(0, |mask, n| {
                if self.threatens(id, b.grid(Cell::new(n % 8, n / 8)), input) {
                    mask | 1u64 << n
                } else {
                    mask
                }
            })
        })
    }
    pub fn summon_legal(&self, kind: Kind, to: Cell, input: &Input) -> bool {
        if ![Kind::Pawn, Kind::Rook, Kind::Bishop, Kind::Knight].contains(&kind)
            || self.occupied(to, 4_294_967_295, input)
            || input
                .fighters
                .iter()
                .any(|f| f.hp > 0 && f.visible && f.cell() == to)
            || !self.clear_segment(to, to, input)
        {
            return false;
        }
        let (x, y) = to.world();
        std::iter::once(input.owner)
            .chain(input.fighters.iter())
            .any(|f| {
                f.team == input.owner.team
                    && f.hp > 0
                    && f.sight_source
                    && !f.friendly_chess
                    && (f.x - x).pow(2) + (f.y - y).pow(2) <= TILE * TILE
                    && (input.sight)(f.x, f.y, x, y)
            })
    }
    pub fn add_piece(
        &mut self,
        kind: Kind,
        cell: Cell,
        max_hp: i64,
        slot: Option<usize>,
        direction: u8,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.pieces
            .insert(id, Piece::new(id, kind, cell, max_hp, slot, direction));
        if let Some(slot) = slot {
            self.slots[slot] = Some(id);
        }
        self.reindex();
        id
    }
    fn placement(&self, input: &Input, heading: u8, seed: u64) -> Option<(Board, [Cell; 7])> {
        let me = input.owner.cell();
        let (x, y) = me.world();
        let b = Board {
            x,
            y,
            heading: heading % 4,
            serial: self.serial + 1,
        };
        let legal = |c: Cell| {
            let c = b.grid(c);
            let (x, y) = c.world();
            !(input.wall)(x, y)
                && c != me
                && !self
                    .pieces
                    .values()
                    .any(|p| p.slot.is_none() && p.cell == c)
                && !input
                    .fighters
                    .iter()
                    .any(|f| f.team == input.owner.team && f.friendly_chess && f.cell() == c)
        };
        let mut rng = seed;
        let mut shuffle = |v: &mut Vec<Cell>| {
            for i in (1..v.len()).rev() {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                v.swap(i, (rng % (i + 1) as u64) as usize);
            }
        };
        let mut adjacent: Vec<_> = (0..64)
            .map(|n| Cell::new(n % 8, n / 8))
            .filter(|c| b.grid(*c).distance(me) == 1 && legal(*c))
            .collect();
        shuffle(&mut adjacent);
        let q = *adjacent.first()?;
        let mut free: Vec<_> = (0..64)
            .map(|n| Cell::new(n % 8, n / 8))
            .filter(|c| b.grid(*c).distance(me) > 1 && legal(*c))
            .collect();
        shuffle(&mut free);
        let light = free.iter().position(|c| (c.x + c.y) % 2 == 0)?;
        let bi = free.remove(light);
        let dark = free.iter().position(|c| (c.x + c.y) % 2 != 0)?;
        let bj = free.remove(dark);
        if free.len() < 4 {
            return None;
        }
        Some((
            b,
            [
                b.grid(free[0]),
                b.grid(free[1]),
                b.grid(bi),
                b.grid(bj),
                b.grid(free[2]),
                b.grid(free[3]),
                b.grid(q),
            ],
        ))
    }
    fn castle_legal(&self, id: usize, input: &Input) -> bool {
        let Some(b) = self.board else { return false };
        let Some(p) = self.pieces.get(&id) else {
            return false;
        };
        let me = input.owner.cell();
        p.kind == Kind::Rook
            && b.inside(input.owner.x, input.owner.y)
            && b.inside(p.cell.world().0, p.cell.world().1)
            && me.distance(p.cell) <= 3
            && me != p.cell
            && (me.x == p.cell.x || me.y == p.cell.y)
            && self.clear_segment(me, p.cell, input)
            && (1..me.distance(p.cell)).all(|n| {
                !self.occupied(
                    Cell::new(
                        me.x + (p.cell.x - me.x).signum() * n,
                        me.y + (p.cell.y - me.y).signum() * n,
                    ),
                    id,
                    input,
                )
            })
    }
    pub fn begin(&mut self, input: &Input, action: Action, yield_move: bool) -> bool {
        if self.pending.is_some() || input.controlled {
            return false;
        }
        let (from, kind) = match action {
            Action::Deploy { heading } => {
                if input.tick < self.reset_at || self.placement(input, heading, self.rng).is_none()
                {
                    return false;
                }
                (input.owner.cell(), Kind::Rook)
            }
            Action::Move { piece, to } => {
                if !self.legal(piece, to, input) {
                    return false;
                }
                (self.pieces[&piece].cell, self.pieces[&piece].kind)
            }
            Action::Castle { piece } => {
                if input.tick < self.castle_at || !self.castle_legal(piece, input) {
                    return false;
                }
                (input.owner.cell(), Kind::Rook)
            }
            Action::Summon { kind, to, .. } => {
                if input.tick
                    < if kind == Kind::Pawn {
                        self.pawn_at
                    } else {
                        self.develop_at
                    }
                    || !self.summon_legal(kind, to, input)
                {
                    return false;
                }
                (to, kind)
            }
            Action::Promote { piece, kind } => {
                let Some(p) = self.pieces.get_mut(&piece) else {
                    return false;
                };
                if p.kind != Kind::Pawn || kind == Kind::Pawn {
                    return false;
                }
                p.promotion = kind;
                return true;
            }
            Action::Dismiss => {
                self.dismiss();
                return true;
            }
        };
        let duration = if matches!(action, Action::Deploy { .. }) {
            60
        } else {
            self.duration(
                if matches!(action, Action::Move { .. }) {
                    kind
                } else {
                    Kind::Rook
                },
                input.tick,
            )
        };
        let warn = if duration == 60 {
            60
        } else {
            (duration * 3 + 2) / 5
        };
        let multiplier = self
            .reward
            .as_ref()
            .filter(|r| input.tick < r.until && r.charges > 0)
            .map_or(100, |r| match r.id {
                0 if kind == Kind::Rook => 125,
                3 if matches!(kind, Kind::Knight | Kind::Bishop) => 120,
                7 if matches!(kind, Kind::Rook | Kind::Bishop | Kind::Queen) => 125,
                8 if kind == Kind::Pawn => 150,
                _ => 100,
            });
        let before = self
            .pieces
            .keys()
            .map(|id| {
                (
                    *id,
                    input
                        .fighters
                        .iter()
                        .filter(|f| {
                            f.team != input.owner.team
                                && f.visible
                                && self.threatens(*id, f.cell(), input)
                        })
                        .map(|f| f.id)
                        .collect(),
                )
            })
            .collect();
        self.pending = Some(Pending {
            action,
            start: input.tick,
            commit: input.tick + warn,
            end: input.tick + duration,
            from,
            hit: BTreeSet::new(),
            committed: false,
            yield_move,
            multiplier,
            before,
        });
        true
    }
    pub fn dismiss(&mut self) {
        self.board = None;
        self.outside_since = None;
        self.mate_warning = None;
        self.mate_target = None;
        self.mate_cell = None;
        self.has_move = false;
        if self
            .challenge
            .as_ref()
            .is_some_and(|c| c.board_serial.is_some() && !c.done && !c.failed)
        {
            if let Some(c) = &mut self.challenge {
                c.failed = true;
                c.failure = 6;
                c.finished = self.last_tick;
            }
        }
    }
    pub fn on_dead(&mut self) -> Vec<Cmd> {
        let out = self
            .pieces
            .keys()
            .map(|id| Cmd::Despawn { piece: *id })
            .collect();
        self.pieces.clear();
        self.reindex();
        self.slots = [None; 7];
        self.pending = None;
        self.challenge = None;
        self.reward = None;
        self.projections.clear();
        self.field_target = None;
        self.awaiting.clear();
        self.confirmed.clear();
        self.positions.clear();
        self.owner_position = None;
        self.dismiss();
        out
    }
    pub fn spawn_failed(&mut self, id: usize, tick: usize) {
        if let Some(p) = self.pieces.remove(&id) {
            if p.slot.is_none() {
                if p.kind == Kind::Pawn {
                    self.pawn_at = tick;
                } else {
                    self.develop_at = tick;
                }
            } else if let Some(slot) = p.slot {
                self.slots[slot] = None;
            }
        }
        self.last_failure = 3;
        self.reindex();
    }
    fn deploy(&mut self, input: &Input, heading: u8, out: &mut Vec<Cmd>) -> bool {
        let Some((b, cells)) = self.placement(input, heading, self.rng) else {
            return false;
        };
        self.roll(4_294_967_295);
        self.serial += 1;
        self.board = Some(b);
        self.outside_since = None;
        self.has_move = false;
        self.mate_warning = None;
        self.reset_at = input.tick + RESET;
        self.stats.resets += 1;
        self.challenge = None;
        for slot in 0..7 {
            let kind = PIECE_KINDS[slot];
            let max_hp = (input.owner.max_hp * kind.hp_pct() / 100).max(1);
            let id = if let Some(id) = self.slots[slot].filter(|id| self.pieces.contains_key(id)) {
                let p = self.pieces.get_mut(&id).unwrap();
                p.cell = cells[slot];
                p.max_hp = max_hp;
                p.hp = if slot == 6 { max_hp } else { p.hp.min(max_hp) };
                p.exposed = false;
                p.yielding = false;
                id
            } else {
                self.add_piece(kind, cells[slot], max_hp, Some(slot), heading)
            };
            let p = &self.pieces[&id];
            let (x, y) = p.cell.world();
            out.push(Cmd::Spawn {
                piece: id,
                x,
                y,
                hp: p.hp,
            });
            if let Some(reward) = self
                .reward
                .as_ref()
                .filter(|r| r.id == 4 && input.tick < r.until)
            {
                out.push(Cmd::Buff {
                    target: Some(id),
                    kind: 4,
                    ticks: reward.until - input.tick,
                });
            }
        }
        self.reindex();
        self.project(input, true);
        self.select_challenge(input);
        self.label(1, input, out);
        true
    }
}
pub fn forward(direction: u8) -> (i32, i32) {
    [(0, 1), (1, 0), (0, -1), (-1, 0)][direction as usize % 4]
}

impl Core {
    fn observe(&mut self, input: &Input) {
        for p in self.pieces.values_mut().filter(|p| p.kind == Kind::Pawn) {
            if p.intercept.is_some_and(|(id, c, t)| {
                input.tick > t
                    || !input
                        .fighters
                        .iter()
                        .any(|f| f.id == id && f.hp > 0 && f.visible && f.cell() == c)
            }) {
                p.intercept = None;
            }
            let (fx, fy) = forward(p.direction);
            for f in input
                .fighters
                .iter()
                .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && f.targetable)
            {
                if let Some((old, tick)) = self.positions.get(&f.id) {
                    let now = f.cell();
                    let local = |c: Cell| {
                        let (dx, dy) = (c.x - p.cell.x, c.y - p.cell.y);
                        (dx * fx + dy * fy, dx * fy - dy * fx)
                    };
                    let (a, s) = local(*old);
                    let (b, z) = local(now);
                    if input.tick == tick + 1
                        && old.distance(now) <= 1
                        && a == 1
                        && b == 1
                        && s != z
                        && z.abs() == 1
                        && s * z <= 0
                    {
                        let mid = Cell::new(p.cell.x + fx, p.cell.y + fy);
                        if p.cell.distance(now) == 1 && segment_cells(*old, now).contains(&mid) {
                            p.intercept = Some((f.id, now, input.tick + 60));
                        }
                    }
                }
            }
        }
        for f in input.fighters.iter().filter(|f| f.visible && f.hp > 0) {
            self.positions.insert(f.id, (f.cell(), input.tick));
        }
        self.positions.retain(|_, (_, t)| *t == input.tick);
    }
    fn confirm_hits(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        for h in input.hits {
            let Some((expiry, discovered)) = self.awaiting.remove(&(h.command, h.piece, h.target))
            else {
                continue;
            };
            if input.tick > expiry || h.amount <= 0 || !self.pieces.contains_key(&h.piece) {
                continue;
            }
            let champion = input
                .fighters
                .iter()
                .any(|f| f.id == h.target && f.champion && f.team != input.owner.team);
            if champion {
                if let Some(p) = self.pieces.get_mut(&h.piece) {
                    p.qualified = true;
                }
            }
            self.confirmed
                .insert((h.command, h.piece, h.target), discovered);
        }
        self.awaiting.retain(|_, (until, _)| input.tick <= *until);
        self.flush_hits(input, out);
    }
    fn flush_hits(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        let ready: Vec<_> = self
            .confirmed
            .iter()
            .filter(|((command, _, _), _)| {
                self.pending.as_ref().is_none_or(|p| p.start != *command)
            })
            .map(|(key, discovered)| (*key, *discovered))
            .collect();
        for ((command, piece, target), discovered) in ready {
            self.confirmed.remove(&(command, piece, target));
            if !self.pieces.contains_key(&piece) {
                continue;
            }
            if self
                .visible(input, target)
                .is_some_and(|f| f.champion && self.board.is_none_or(|b| !b.inside(f.x, f.y)))
            {
                self.field_target = Some(target);
            }
            if self.challenge.is_none() {
                if self.board.is_none() {
                    self.project(input, true);
                }
                self.select_challenge(input);
            }
            self.challenge_event(input, 1, piece, Some(target), out);
            if discovered {
                self.challenge_event(input, 4, piece, Some(target), out);
                self.label(5, input, out);
            }
        }
    }
    fn promote_ready(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        let ready: Vec<_> = self
            .pieces
            .values()
            .filter(|p| p.kind == Kind::Pawn && p.ranks >= 6 && p.qualified && !p.yielding)
            .map(|p| p.id)
            .collect();
        for id in ready {
            let p = self.pieces.get_mut(&id).unwrap();
            p.kind = p.promotion;
            let max = (input.owner.max_hp * p.kind.hp_pct() / 100).max(1);
            p.hp = (p.hp * max / p.max_hp).max(1);
            p.max_hp = max;
            self.stats.promotions += 1;
            out.push(Cmd::Appearance { piece: id });
            self.challenge_event(input, 6, id, None, out);
        }
    }
    fn resolve_move(&mut self, input: &Input, p: &mut Pending, out: &mut Vec<Cmd>) {
        let Action::Move { piece, to } = p.action else {
            return;
        };
        let Some(unit) = self.pieces.get(&piece) else {
            return;
        };
        let kind = unit.kind;
        let elapsed = input.tick.saturating_sub(p.commit).min(p.end - p.commit);
        let duration = (p.end - p.commit).max(1);
        let (ax, ay) = p.from.world();
        let (bx, by) = to.world();
        let x = ax + (bx - ax) * elapsed as i64 / duration as i64;
        let y = ay + (by - ay) * elapsed as i64 / duration as i64;
        out.push(Cmd::Move { piece, x, y });
        if p.yield_move {
            return;
        }
        let at = Cell::at(x, y);
        let knight = kind == Kind::Knight;
        let pawn = kind == Kind::Pawn;
        let (fx, fy) = forward(unit.direction);
        let pawn_attack = (to.x - p.from.x) * fy - (to.y - p.from.y) * fx != 0;
        let reached = p.from.distance(at);
        let ep = unit.intercept;
        for f in input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.targetable && f.visible)
        {
            let cell = f.cell();
            let hit = if knight || pawn {
                input.tick >= p.end
                    && cell == to
                    && (!pawn || pawn_attack)
                    && ep.is_none_or(|(id, c, until)| c != to || f.id == id && input.tick <= until)
            } else {
                segment_cells(p.from, to)
                    .into_iter()
                    .skip(1)
                    .take(reached as usize)
                    .any(|c| c == cell)
            };
            if !hit || !p.hit.insert(f.id) {
                continue;
            }
            let inside = self
                .board
                .is_some_and(|b| b.inside(x, y) && b.inside(f.x, f.y));
            let amount =
                (kind.damage(input.owner.ap) * p.multiplier * if inside { 100 } else { 50 }
                    / (self.army_denominator() * 100))
                    .max(1);
            self.stats.damage += amount;
            out.push(Cmd::Damage {
                command: p.start,
                piece,
                target: f.id,
                amount,
            });
            let discovered = self
                .projected(Role::King)
                .and_then(|id| self.visible(input, id))
                .is_some_and(|king| {
                    self.pieces.values().any(|other| {
                        other.id != piece
                            && matches!(other.kind, Kind::Bishop | Kind::Rook | Kind::Queen)
                            && !p
                                .before
                                .get(&other.id)
                                .is_some_and(|ids| ids.contains(&king.id))
                            && {
                                let mut view = self.clone();
                                if let Some(moved) = view.pieces.get_mut(&piece) {
                                    moved.cell = to;
                                }
                                view.reindex();
                                view.threatens(other.id, king.cell(), input)
                            }
                    })
                });
            self.awaiting
                .insert((p.start, piece, f.id), (input.tick + 120, discovered));
        }
    }
    fn finish_move(&mut self, input: &Input, p: &Pending, out: &mut Vec<Cmd>) {
        let Action::Move { piece, to } = p.action else {
            return;
        };
        let Some(unit) = self.pieces.get_mut(&piece) else {
            return;
        };
        unit.cell = to;
        unit.moved = true;
        unit.yielding = false;
        if !p.yield_move && unit.kind == Kind::Pawn {
            let (fx, fy) = forward(unit.direction);
            let ranks = ((to.x - p.from.x) * fx + (to.y - p.from.y) * fy).max(0);
            unit.ranks = unit.ranks.saturating_add(ranks as u8);
        }
        self.reindex();
        if p.yield_move {
            return;
        }
        self.has_move = true;
        let kind = self.pieces[&piece].kind;
        self.stats.commands[kind.code()] += 1;
        self.challenge_event(input, 0, piece, None, out);
        if kind == Kind::Pawn {
            self.challenge_event(input, 5, piece, None, out);
        }
        if kind == Kind::Knight
            && input
                .fighters
                .iter()
                .filter(|f| {
                    f.team != input.owner.team
                        && f.hp > 0
                        && f.visible
                        && f.champion
                        && self.threatens(piece, f.cell(), input)
                })
                .count()
                >= 2
        {
            self.label(4, input, out);
        }
        if let Some(k) = self
            .projected(Role::King)
            .and_then(|id| self.visible(input, id))
        {
            let checking = self
                .pieces
                .keys()
                .any(|id| self.threatens(*id, k.cell(), input));
            if checking {
                self.label(2, input, out);
            }
            for f in input
                .fighters
                .iter()
                .filter(|f| f.id != k.id && f.team != input.owner.team && f.visible && f.hp > 0)
            {
                let view = &*self;
                let is_pin = self.pieces.values().any(|slider| {
                    matches!(slider.kind, Kind::Rook | Kind::Bishop | Kind::Queen)
                        && segment_cells(slider.cell, k.cell()).contains(&f.cell())
                        && slider.cell.distance(f.cell()) < slider.cell.distance(k.cell())
                        && view.threatens(slider.id, k.cell(), input)
                });
                if is_pin {
                    self.label(11, input, out);
                    break;
                }
            }
        }
    }
    pub fn step(&mut self, input: &Input) -> Vec<Cmd> {
        if self.last_tick.is_some_and(|t| input.tick <= t) {
            return vec![];
        }
        let previous = self.last_tick.unwrap_or(input.tick);
        let owner_previous = self.owner_position.replace((input.owner.x, input.owner.y));
        self.last_tick = Some(input.tick);
        let mut out = vec![];
        if input.owner.hp <= 0 {
            return self.on_dead();
        }
        self.last_failure = 0;
        self.reindex();
        self.confirm_hits(input, &mut out);
        for c in input.captures {
            self.capture(input, *c, &mut out);
        }
        for (id, hp) in input.piece_hp {
            if let Some(p) = self.pieces.get_mut(id) {
                p.hp = (*hp).clamp(0, p.max_hp);
            }
        }
        let dead: Vec<_> = self
            .pieces
            .values()
            .filter(|p| p.hp <= 0)
            .map(|p| p.id)
            .collect();
        for id in dead {
            self.capture(
                input,
                Capture {
                    piece: id,
                    by: None,
                    combat: true,
                },
                &mut out,
            );
        }
        self.reindex();
        self.observe(input);
        self.project(input, false);
        if let Some(b) = self.board {
            if b.inside(input.owner.x, input.owner.y) {
                self.outside_since = None;
            } else {
                let since = *self.outside_since.get_or_insert(input.tick);
                if input.tick >= since + 300 {
                    self.dismiss();
                }
            }
        }
        if let Some(r) = &self.reward {
            if input.tick >= r.until {
                self.reward = None;
            }
        }
        self.immune.retain(|_, until| input.tick < *until);
        if let Some(c) = self.challenge.as_mut() {
            if !c.done && !c.failed {
                let lost = c.target.is_some_and(|id| {
                    !input.fighters.iter().any(|f| {
                        f.id == id
                            && f.hp > 0
                            && f.visible
                            && f.targetable
                            && c.board_serial
                                .is_none_or(|_| self.board.is_some_and(|b| b.inside(f.x, f.y)))
                    })
                });
                let lost_queen = c.queen_target.is_some_and(|id| {
                    GAMES[c.id].queen_target
                        && !input
                            .fighters
                            .iter()
                            .any(|f| f.id == id && f.hp > 0 && f.visible)
                });
                if lost || lost_queen {
                    c.failed = true;
                    c.failure = if lost { 2 } else { 3 };
                    c.finished = Some(input.tick);
                } else if c
                    .started
                    .is_some_and(|start| input.tick > start + GAMES[c.id].window)
                {
                    c.failed = true;
                    c.failure = 1;
                    c.finished = Some(input.tick);
                }
            }
        }
        if self
            .challenge
            .as_ref()
            .is_some_and(|c| c.finished.is_some_and(|t| input.tick >= t + 120))
        {
            self.challenge = None;
            if self.board.is_none() {
                self.project(input, true);
            }
            self.select_challenge(input);
        }
        let owner_now = input.owner.cell();
        if let Some(c) = &self.challenge {
            let old = c.owner_last;
            let dt = input.tick.saturating_sub(previous).max(1) as i64;
            if old != owner_now
                && old.distance(owner_now) <= 1
                && owner_previous.is_some_and(|(ax, ay)| {
                    (input.owner.x - ax).abs().max((input.owner.y - ay).abs()) <= 6000 * dt
                })
            {
                self.challenge_event(input, 7, 4_294_967_295, None, &mut out);
            }
        }
        for piece in self.pieces.values_mut() {
            piece.yielding = piece.cell == owner_now;
        }
        if let Some(mut p) = self.pending.take() {
            if !p.committed && input.controlled {
                self.stats.cancelled += 1;
                self.label(9, input, &mut out);
            } else {
                if !p.committed && input.tick >= p.commit {
                    let valid = match p.action {
                        Action::Move { piece, to } => self.legal(piece, to, input),
                        Action::Castle { piece } => self.castle_legal(piece, input),
                        Action::Summon { kind, to, .. } => self.summon_legal(kind, to, input),
                        Action::Deploy { heading } => {
                            self.placement(input, heading, self.rng).is_some()
                        }
                        _ => true,
                    };
                    if !valid {
                        self.stats.cancelled += 1;
                        self.last_failure = 1;
                        out.push(Cmd::Failure { reason: 1 });
                    } else {
                        p.committed = true;
                        let kind = match p.action {
                            Action::Move { piece, .. } => {
                                self.pieces.get(&piece).map_or(Kind::Rook, |p| p.kind)
                            }
                            Action::Summon { kind, .. } => kind,
                            _ => Kind::Rook,
                        };
                        if !p.yield_move
                            && !matches!(p.action, Action::Deploy { .. })
                            && (matches!(p.action, Action::Move { .. })
                                || self.reward.as_ref().is_some_and(|r| r.id == 6))
                        {
                            p.multiplier = self.consume_reward(input, kind, &mut out);
                        }
                    }
                }
                if p.committed {
                    match p.action {
                        Action::Move { piece, to } => {
                            if self.occupied(to, piece, input) {
                                let (x, y) = p.from.world();
                                out.push(Cmd::Move { piece, x, y });
                                self.stats.cancelled += 1;
                            } else {
                                self.resolve_move(input, &mut p, &mut out);
                                if input.tick >= p.end {
                                    self.finish_move(input, &p, &mut out);
                                } else {
                                    self.pending = Some(p);
                                }
                            }
                        }
                        Action::Deploy { heading } => {
                            self.deploy(input, heading, &mut out);
                        }
                        Action::Castle { piece } => {
                            if input.tick >= p.end {
                                if !self.castle_legal(piece, input) {
                                    self.stats.cancelled += 1;
                                    self.last_failure = 1;
                                    out.push(Cmd::Failure { reason: 1 });
                                } else {
                                    if let Some(unit) = self.pieces.get_mut(&piece) {
                                        let to = unit.cell;
                                        unit.cell = input.owner.cell();
                                        let (x, y) = unit.cell.world();
                                        out.push(Cmd::Move { piece, x, y });
                                        let (x, y) = to.world();
                                        out.push(Cmd::OwnerPos { x, y });
                                        self.owner_position = Some((x, y));
                                        if let Some(c) = self.challenge.as_mut() {
                                            c.owner_last = to;
                                        }
                                        self.castle_at = input.tick + CASTLE_CD;
                                        self.stats.casts += 1;
                                        out.push(Cmd::Shield {
                                            target: None,
                                            amount: 80 + input.owner.ap / 5,
                                            ticks: 60,
                                        });
                                        self.label(10, input, &mut out);
                                        self.reindex();
                                    }
                                }
                            } else {
                                self.pending = Some(p);
                            }
                        }
                        Action::Summon {
                            kind,
                            to,
                            direction,
                        } => {
                            if input.tick >= p.end {
                                if !self.summon_legal(kind, to, input) {
                                    self.last_failure = 1;
                                    self.stats.cancelled += 1;
                                    out.push(Cmd::Failure { reason: 1 });
                                    self.mate_warning = None;
                                } else {
                                    let id = self.add_piece(
                                        kind,
                                        to,
                                        (input.owner.max_hp * kind.hp_pct() / 100).max(1),
                                        None,
                                        direction,
                                    );
                                    let (x, y) = to.world();
                                    out.push(Cmd::Spawn {
                                        piece: id,
                                        x,
                                        y,
                                        hp: self.pieces[&id].hp,
                                    });
                                    if kind == Kind::Pawn {
                                        self.pawn_at = input.tick + 360;
                                    } else {
                                        self.develop_at = input.tick + 600;
                                    }
                                    self.stats.summons += 1;
                                    self.has_move = false;
                                    self.mate_warning = None;
                                    if self
                                        .reward
                                        .as_ref()
                                        .is_some_and(|r| r.id == 4 && input.tick < r.until)
                                    {
                                        out.push(Cmd::Buff {
                                            target: Some(id),
                                            kind: 4,
                                            ticks: self.reward.as_ref().unwrap().until - input.tick,
                                        });
                                    }
                                }
                            } else {
                                self.pending = Some(p);
                            }
                        }
                        _ => {}
                    }
                } else if input.tick < p.commit {
                    self.pending = Some(p);
                }
            }
        }
        self.flush_hits(input, &mut out);
        self.promote_ready(input, &mut out);
        self.checkmate(input, &mut out);
        if input.tick.saturating_sub(self.last_fx) >= 30 {
            if let Some(b) = self.board {
                let (x, y) = b.world(Cell::new(0, 0));
                let (z, w) = b.world(Cell::new(7, 7));
                let fading = self.outside_since.is_some_and(|t| input.tick >= t + 240);
                self.fx(
                    input.tick,
                    &format!("board{}{}", b.heading, if fading { "_fade" } else { "" }),
                    (x + z) / 2,
                    (y + w) / 2,
                    30,
                    true,
                    &mut out,
                );
            }
            self.last_fx = input.tick;
        }
        if self.pending.is_none() && !input.controlled {
            let yielding = self.pieces.values().find(|p| p.yielding).map(|p| p.id);
            let forced = yielding.and_then(|id| {
                let mut candidates = self.destinations(id, input);
                candidates.sort_by_key(|c| (self.pieces[&id].cell.distance(*c), c.x, c.y));
                candidates
                    .first()
                    .copied()
                    .map(|to| Action::Move { piece: id, to })
            });
            if let Some(action) = forced {
                self.begin(input, action, true);
            } else if let Some(action) = input.action {
                if !self.begin(input, action, false) {
                    self.last_failure = 2;
                    out.push(Cmd::Failure { reason: 2 });
                }
            } else if input.automatic {
                if let Some(action) = self.plan(input) {
                    self.begin(input, action, false);
                }
            }
        }
        if let Some(p) = &self.pending {
            if input.tick == p.start {
                if let Action::Move { to, .. } | Action::Summon { to, .. } = p.action {
                    let (x, y) = to.world();
                    self.fx(
                        input.tick,
                        if p.end - p.start == 45 {
                            "warn45"
                        } else {
                            "warn30"
                        },
                        x,
                        y,
                        p.commit - p.start,
                        false,
                        &mut out,
                    );
                }
            }
        }
        out
    }
    pub fn destinations(&self, id: usize, input: &Input) -> Vec<Cell> {
        let Some(p) = self.pieces.get(&id) else {
            return vec![];
        };
        let offsets: Vec<_> = match p.kind {
            Kind::Knight => vec![
                (1, 2),
                (2, 1),
                (-1, 2),
                (-2, 1),
                (1, -2),
                (2, -1),
                (-1, -2),
                (-2, -1),
            ],
            Kind::Pawn => {
                let (x, y) = forward(p.direction);
                vec![(x, y), (x * 2, y * 2), (x + y, y - x), (x - y, y + x)]
            }
            _ => DIRS
                .iter()
                .flat_map(|(x, y)| (1..=7).map(move |n| (x * n, y * n)))
                .collect(),
        };
        offsets
            .into_iter()
            .map(|(x, y)| Cell::new(p.cell.x + x, p.cell.y + y))
            .filter(|to| self.legal(id, *to, input))
            .collect()
    }
    pub fn warning_cells(&self) -> Vec<Cell> {
        if let Some(cell) = self.mate_cell.filter(|_| self.mate_warning.is_some()) {
            return std::iter::once(cell)
                .chain(DIRS.map(|(x, y)| Cell::new(cell.x + x, cell.y + y)))
                .collect();
        }
        self.pending
            .as_ref()
            .map_or_else(Vec::new, |p| match p.action {
                Action::Move { piece, to } => {
                    if self
                        .pieces
                        .get(&piece)
                        .is_some_and(|p| matches!(p.kind, Kind::Knight | Kind::Pawn))
                    {
                        vec![to]
                    } else {
                        segment_cells(p.from, to).into_iter().skip(1).collect()
                    }
                }
                Action::Summon { to, .. } => vec![to],
                _ => vec![],
            })
    }
}
pub fn segment_cells(a: Cell, b: Cell) -> Vec<Cell> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let n = dx.abs().max(dy.abs());
    if n == 0 {
        return vec![a];
    }
    (0..=n)
        .map(|i| Cell::new(a.x + dx * i / n, a.y + dy * i / n))
        .collect()
}

impl Core {
    fn move_score(&self, id: usize, to: Cell, input: &Input) -> i64 {
        let Some(p) = self.pieces.get(&id) else {
            return i64::MIN / 4;
        };
        let mut score = 0;
        for f in input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && f.targetable)
        {
            let hit = if matches!(p.kind, Kind::Knight | Kind::Pawn) {
                f.cell() == to
            } else {
                segment_cells(p.cell, to)
                    .into_iter()
                    .skip(1)
                    .any(|c| c == f.cell())
            };
            if hit {
                score += p.kind.damage(input.owner.ap) * if f.champion { 8 } else { 2 };
            }
            if f.champion {
                score += 24 - i64::from(to.distance(f.cell())).min(24) * 3;
                if to == f.cell() {
                    score -= 40;
                }
            }
        }
        if self
            .board
            .is_some_and(|b| b.inside(to.world().0, to.world().1))
        {
            score += 30;
        }
        if p.kind == Kind::Pawn {
            let (x, y) = forward(p.direction);
            score += i64::from(((to.x - p.cell.x) * x + (to.y - p.cell.y) * y).max(0)) * 15;
            if p.ranks >= 5 && p.qualified {
                score += 150;
            }
        }
        if p.hp * 3 < p.max_hp {
            score -= input
                .fighters
                .iter()
                .filter(|f| {
                    f.team != input.owner.team
                        && f.visible
                        && f.champion
                        && to.distance(f.cell()) <= 2
                })
                .count() as i64
                * 80;
        }
        if self
            .challenge
            .as_ref()
            .is_some_and(|c| !c.failed && !c.done && c.bindings.contains(&Some(id)))
        {
            score += 25;
        }
        score
    }
    pub fn plan(&mut self, input: &Input) -> Option<Action> {
        self.last_plan_candidates = 0;
        if input.controlled || self.pending.is_some() {
            return None;
        }
        let foes: Vec<_> = input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && f.targetable)
            .collect();
        let nearest = foes.iter().min_by_key(|f| {
            (
                (f.x - input.owner.x).pow(2) + (f.y - input.owner.y).pow(2),
                f.id,
            )
        });
        if let Some(foe) = nearest {
            if input.tick >= self.reset_at
                && (self.board.is_none() || self.slots.iter().flatten().count() < 4)
                && input.owner.cell().distance(foe.cell()) <= 7
            {
                let heading = if (foe.x - input.owner.x).abs() > (foe.y - input.owner.y).abs() {
                    if foe.x >= input.owner.x {
                        1
                    } else {
                        3
                    }
                } else if foe.y >= input.owner.y {
                    0
                } else {
                    2
                };
                if self.placement(input, heading, self.rng).is_some() {
                    return Some(Action::Deploy { heading });
                }
            }
        }
        if input.owner.hp * 3 < input.owner.max_hp && input.tick >= self.castle_at {
            if let Some(p) = self.pieces.values().find(|p| {
                p.kind == Kind::Rook
                    && self.castle_legal(p.id, input)
                    && foes.iter().all(|f| {
                        !f.champion
                            || p.cell.distance(f.cell()) > input.owner.cell().distance(f.cell())
                    })
            }) {
                return Some(Action::Castle { piece: p.id });
            }
        }
        if self
            .challenge
            .as_ref()
            .is_some_and(|c| c.id == 10 && !c.done && !c.failed)
        {
            if let Some(p) = self.pieces.values().find(|p| {
                p.kind == Kind::Pawn && p.qualified && p.ranks >= 5 && p.promotion != Kind::Rook
            }) {
                return Some(Action::Promote {
                    piece: p.id,
                    kind: Kind::Rook,
                });
            }
        }
        let ids: Vec<_> = self.pieces.keys().copied().collect();
        let count = ids.len().min(16);
        let tactical_ids: Vec<_> = (0..count)
            .map(|i| ids[(self.planner_cursor + i) % ids.len()])
            .collect();
        let mut roots = vec![];
        for i in 0..count {
            let id = tactical_ids[i];
            let mut destinations = self.destinations(id, input);
            destinations.sort_by_key(|to| {
                (
                    foes.iter()
                        .map(|f| to.distance(f.cell()))
                        .min()
                        .unwrap_or(0),
                    *to,
                )
            });
            if let Some((score, to)) = self
                .candidate_scores(id, destinations.into_iter().take(2), input)
                .into_iter()
                .max_by_key(|(s, c)| (*s, std::cmp::Reverse(*c)))
            {
                roots.push((score, Action::Move { piece: id, to }));
            }
        }
        self.planner_cursor = (self.planner_cursor + count.max(1)) % ids.len().max(1);
        // Summon candidates are anchored to observed allies, not the enemy's hidden position.
        if input.tick >= self.pawn_at || input.tick >= self.develop_at {
            let mut cells = BTreeSet::new();
            for ally in std::iter::once(input.owner)
                .chain(input.fighters.iter())
                .filter(|f| {
                    f.team == input.owner.team && f.hp > 0 && f.sight_source && !f.friendly_chess
                })
            {
                for (dx, dy) in DIRS {
                    cells.insert(Cell::new(ally.cell().x + dx, ally.cell().y + dy));
                }
            }
            let mut cells: Vec<_> = cells.into_iter().collect();
            cells.sort_by_key(|c| {
                (
                    foes.iter()
                        .map(|f| c.distance(f.cell()))
                        .min()
                        .unwrap_or(c.distance(input.owner.cell())),
                    *c,
                )
            });
            for to in cells.into_iter().take(8) {
                let direction = nearest.map_or(0, |f| {
                    let (dx, dy) = (f.cell().x - to.x, f.cell().y - to.y);
                    if dx.abs() > dy.abs() {
                        if dx >= 0 {
                            1
                        } else {
                            3
                        }
                    } else if dy >= 0 {
                        0
                    } else {
                        2
                    }
                });
                let develop =
                    [Kind::Rook, Kind::Knight, Kind::Bishop][(self.stats.summons + self.rank) % 3];
                for kind in [Kind::Pawn, develop] {
                    if input.tick
                        >= if kind == Kind::Pawn {
                            self.pawn_at
                        } else {
                            self.develop_at
                        }
                        && self.summon_legal(kind, to, input)
                    {
                        let score = if self.pieces.len() < 7 { 150 } else { 30 }
                            - self.active_count().saturating_sub(7) as i64 * 5
                            + if foes.iter().any(|f| to.distance(f.cell()) <= 3) {
                                65
                            } else {
                                0
                            };
                        roots.push((
                            score,
                            Action::Summon {
                                kind,
                                to,
                                direction,
                            },
                        ));
                        self.last_plan_candidates += 1;
                    }
                }
            }
        }
        roots.sort_by_key(|(score, action)| (std::cmp::Reverse(*score), action_key(*action)));
        roots.truncate(8);
        if roots.is_empty() {
            return None;
        }
        let mut beam: Vec<_> = roots
            .into_iter()
            .map(|(score, action)| (score, action, self.clone()))
            .collect();
        let mut budget = self.last_plan_candidates;
        let horizon = 1 + self.rank * 3 / 7;
        for depth in 1..horizon {
            let mut next = vec![];
            let allowance = (64 - budget) / (horizon - depth).max(1);
            let mut remaining = allowance;
            for (branch, (score, first, mut view)) in beam.iter().take(8).cloned().enumerate() {
                if let Action::Move { piece, to } = first {
                    if depth == 1 {
                        if let Some(p) = view.pieces.get_mut(&piece) {
                            p.cell = to;
                        }
                        view.reindex();
                    }
                }
                let mut future = vec![];
                let quota = remaining / (8 - branch).min(beam.len() - branch).max(1);
                let tactical: Vec<_> = tactical_ids
                    .iter()
                    .cycle()
                    .skip(depth + branch)
                    .take(count)
                    .copied()
                    .collect();
                for id in tactical.into_iter().take(quota) {
                    let mut destinations = view.destinations(id, input);
                    destinations.sort_by_key(|to| {
                        (
                            foes.iter()
                                .map(|f| to.distance(f.cell()))
                                .min()
                                .unwrap_or(0),
                            *to,
                        )
                    });
                    if let Some((value, to)) = view
                        .candidate_scores(id, destinations.into_iter().take(1), input)
                        .into_iter()
                        .max_by_key(|(s, c)| (*s, std::cmp::Reverse(*c)))
                    {
                        future.push((value, id, to));
                        budget += 1;
                        remaining = remaining.saturating_sub(1);
                    }
                }
                future.sort_by_key(|(s, id, to)| (std::cmp::Reverse(*s), *id, *to));
                for (value, id, to) in future.into_iter().take(2) {
                    let mut child = view.clone();
                    child.pieces.get_mut(&id).unwrap().cell = to;
                    child.reindex();
                    next.push((score + value / (depth as i64 + 2), first, child));
                }
            }
            if next.is_empty() {
                break;
            }
            next.sort_by_key(|(s, a, _)| (std::cmp::Reverse(*s), action_key(*a)));
            next.truncate(8);
            beam = next;
        }
        self.last_plan_candidates = budget;
        if beam.is_empty() {
            return None;
        }
        beam.sort_by_key(|(s, a, _)| (std::cmp::Reverse(*s), action_key(*a)));
        let mistake = self.roll(100) < [28, 22, 17, 12, 8, 5, 2, 0][self.rank];
        let index = if mistake {
            self.roll(beam.len().min(3))
        } else {
            0
        };
        Some(beam[index].1)
    }
    fn candidate_scores(
        &mut self,
        id: usize,
        cells: impl Iterator<Item = Cell>,
        input: &Input,
    ) -> Vec<(i64, Cell)> {
        cells
            .map(|to| {
                self.last_plan_candidates += 1;
                (self.move_score(id, to, input), to)
            })
            .collect()
    }
    pub fn json(&self, input: &Input, commands: &[Cmd]) -> String {
        let mut s = String::new();
        let board = self.board.map_or_else(
            || "null".into(),
            |b| {
                format!(
                    "{{\"x\":{},\"y\":{},\"heading\":{},\"serial\":{}}}",
                    b.x, b.y, b.heading, b.serial
                )
            },
        );
        let _=write!(s,"{{\"version\":2,\"tick\":{},\"board\":{},\"outsideSince\":{},\"resetAt\":{},\"pawnAt\":{},\"developAt\":{},\"castleAt\":{},\"mateAt\":{},\"fieldMateAt\":{},\"rewardAt\":{},\"armyDenominator\":{},\"lastFailure\":{},\"mateWarning\":{},\"mateTarget\":{},\"mateBoard\":{},\"glowUntil\":{},\"glow\":{:?},\"boardSlots\":[{}],\"pieces\":[",input.tick,board,option_num(self.outside_since),self.reset_at,self.pawn_at,self.develop_at,self.castle_at,self.mate_at,self.field_mate_at,self.reward_at,self.army_denominator(),self.last_failure,option_num(self.mate_warning),option_num(self.mate_target),self.mate_board,self.glow_until,self.glow.iter().collect::<Vec<_>>(),self.slots.iter().map(|s|option_num(*s)).collect::<Vec<_>>().join(","));
        for (n, p) in self.pieces.values().enumerate() {
            if n > 0 {
                s.push(',');
            }
            let (x, y) = p.cell.world();
            let intercept = p.intercept.map_or_else(
                || "null".into(),
                |(id, c, t)| {
                    format!(
                        "{{\"target\":{id},\"x\":{},\"y\":{},\"until\":{t}}}",
                        c.x, c.y
                    )
                },
            );
            let _=write!(s,"{{\"id\":{},\"kind\":{},\"x\":{},\"y\":{},\"worldX\":{},\"worldY\":{},\"hp\":{},\"maxHp\":{},\"slot\":{},\"generation\":{},\"direction\":{},\"moved\":{},\"ranks\":{},\"qualified\":{},\"promotion\":{},\"exposed\":{},\"yielding\":{},\"intercept\":{},\"weakened\":{}}}",p.id,p.kind.code(),p.cell.x,p.cell.y,x,y,p.hp,p.max_hp,option_num(p.slot),p.generation,p.direction,p.moved,p.ranks,p.qualified,p.promotion.code(),p.exposed,p.yielding,intercept,self.board.is_none_or(|b|!b.inside(x,y)));
        }
        s.push_str("],\"projections\":[");
        for (n, p) in self.projections.iter().enumerate() {
            if n > 0 {
                s.push(',');
            }
            let _ = write!(s, "{{\"id\":{},\"role\":{}}}", p.id, p.role.code());
        }
        s.push_str("],\"coverage\":{");
        for (n, id) in self.pieces.keys().enumerate() {
            if n > 0 {
                s.push(',');
            }
            let _ = write!(s, "\"{}\":\"{:016x}\"", id, self.coverage(*id, input));
        }
        s.push_str("},\"warning\":[");
        for (n, c) in self.warning_cells().iter().enumerate() {
            if n > 0 {
                s.push(',');
            }
            let _ = write!(s, "[{},{}]", c.x, c.y);
        }
        s.push_str("],\"pending\":");
        if let Some(p) = &self.pending {
            let (t, id, x, y, h) = action_key(p.action);
            let _=write!(s,"{{\"type\":{},\"piece\":{},\"x\":{},\"y\":{},\"heading\":{},\"start\":{},\"commit\":{},\"end\":{},\"committed\":{},\"yielding\":{}}}",t,id,x,y,h,p.start,p.commit,p.end,p.committed,p.yield_move);
        } else {
            s.push_str("null");
        }
        s.push_str(",\"challenge\":");
        if let Some(c) = &self.challenge {
            let _=write!(s,"{{\"id\":{},\"progress\":{},\"started\":{},\"done\":{},\"failed\":{},\"failure\":{},\"bindings\":[{}],\"target\":{},\"finished\":{}}}",c.id,c.progress,option_num(c.started),c.done,c.failed,c.failure,c.bindings.iter().map(|b|option_num(*b)).collect::<Vec<_>>().join(","),option_num(c.target),option_num(c.finished));
        } else {
            s.push_str("null");
        }
        s.push_str(",\"reward\":");
        if let Some(r) = &self.reward {
            let _ = write!(
                s,
                "{{\"id\":{},\"until\":{},\"charges\":{},\"piece\":{}}}",
                r.id,
                r.until,
                r.charges,
                option_num(r.piece)
            );
        } else {
            s.push_str("null");
        }
        let _=write!(s,",\"stats\":{{\"commands\":{:?},\"captures\":{},\"mates\":{},\"fieldMates\":{},\"rewards\":{},\"resets\":{},\"damage\":{},\"castles\":{},\"cancelled\":{},\"summons\":{},\"promotions\":{}}},\"commands\":[",self.stats.commands,self.stats.captures,self.stats.mates,self.stats.field_mates,self.stats.rewards,self.stats.resets,self.stats.damage,self.stats.casts,self.stats.cancelled,self.stats.summons,self.stats.promotions);
        for (n, c) in commands.iter().enumerate() {
            if n > 0 {
                s.push(',');
            }
            s.push_str(&command_json(c));
        }
        let _ = write!(s, "],\"debugState\":{}}}", quote(&format!("{self:?}")));
        s
    }
}
pub fn action_key(a: Action) -> (usize, usize, i32, i32, u8) {
    match a {
        Action::Deploy { heading } => (0, 0, 0, 0, heading),
        Action::Move { piece, to } => (1, piece, to.x, to.y, 0),
        Action::Castle { piece } => (2, piece, 0, 0, 0),
        Action::Dismiss => (3, 0, 0, 0, 0),
        Action::Summon {
            kind,
            to,
            direction,
        } => (4, kind.code(), to.x, to.y, direction),
        Action::Promote { piece, kind } => (5, piece, kind.code() as i32, 0, 0),
    }
}
fn option_num(n: Option<usize>) -> String {
    n.map_or_else(|| "null".into(), |n| n.to_string())
}
pub fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
pub fn command_json(c: &Cmd) -> String {
    match c{Cmd::Spawn{piece,x,y,hp}=>format!("{{\"type\":\"spawn\",\"piece\":{piece},\"x\":{x},\"y\":{y},\"hp\":{hp}}}"),Cmd::Despawn{piece}=>format!("{{\"type\":\"despawn\",\"piece\":{piece}}}"),Cmd::Move{piece,x,y}=>format!("{{\"type\":\"move\",\"piece\":{piece},\"x\":{x},\"y\":{y}}}"),Cmd::Damage{command,piece,target,amount}=>format!("{{\"type\":\"damage\",\"command\":{command},\"piece\":{piece},\"target\":{target},\"amount\":{amount}}}"),Cmd::Stun{target,ticks}=>format!("{{\"type\":\"stun\",\"target\":{target},\"ticks\":{ticks}}}"),Cmd::Immune{target,ticks}=>format!("{{\"type\":\"immune\",\"target\":{target},\"ticks\":{ticks}}}"),Cmd::Shield{target,amount,ticks}=>format!("{{\"type\":\"shield\",\"piece\":{},\"amount\":{amount},\"ticks\":{ticks}}}",option_num(*target)),Cmd::Buff{target,kind,ticks}=>format!("{{\"type\":\"buff\",\"piece\":{},\"kind\":{kind},\"ticks\":{ticks}}}",option_num(*target)),Cmd::OwnerPos{x,y}=>format!("{{\"type\":\"owner\",\"x\":{x},\"y\":{y}}}"),Cmd::Fx{tag,x,y,ticks}=>format!("{{\"type\":\"fx\",\"tag\":{},\"x\":{x},\"y\":{y},\"ticks\":{ticks}}}",quote(tag)),Cmd::Label{code}=>format!("{{\"type\":\"label\",\"code\":{code}}}"),Cmd::Failure{reason}=>format!("{{\"type\":\"failure\",\"reason\":{reason}}}"),Cmd::Appearance{piece}=>format!("{{\"type\":\"appearance\",\"piece\":{piece}}}")}
}

impl Core {
    pub fn compatible(&self, input: &Input) -> Vec<usize> {
        let mut counts = [0usize; 5];
        for p in self.pieces.values() {
            counts[p.kind.code()] += 1;
        }
        if self
            .projected(Role::King)
            .and_then(|id| self.visible(input, id))
            .is_none()
        {
            return vec![];
        }
        GAMES
            .iter()
            .enumerate()
            .filter(|(id, g)| {
                input.tick >= self.repeat_at[*id]
                    && g.pieces.iter().enumerate().all(|(k, n)| counts[k] >= *n)
                    && match g.pattern {
                        Pattern::Overload => self.projections.iter().any(|p| {
                            matches!(p.role, Role::Bishop | Role::Knight)
                                && self.visible(input, p.id).is_some()
                        }),
                        Pattern::Century => input.fighters.iter().any(|f| {
                            f.team != input.owner.team
                                && f.hp > 0
                                && f.visible
                                && f.targetable
                                && Some(f.id) != self.projected(Role::King)
                        }),
                        _ => true,
                    }
                    && (!g.queen_target
                        || self
                            .projected(Role::Queen)
                            .and_then(|id| self.visible(input, id))
                            .is_some())
            })
            .map(|(id, _)| id)
            .collect()
    }
    fn select_challenge(&mut self, input: &Input) {
        let mut choices = self.compatible(input);
        if choices.len() > 1 {
            choices.retain(|c| Some(*c) != self.previous_challenge);
        }
        if !choices.is_empty() {
            let id = choices[self.roll(choices.len())];
            let mut c = Challenge::new(id);
            c.target = self.projected(Role::King);
            c.queen_target = self.projected(Role::Queen);
            c.board_serial = self.board.map(|b| b.serial);
            c.owner_last = input.owner.cell();
            self.challenge = Some(c);
            self.previous_challenge = Some(id);
        }
    }
    pub fn choose_challenge(&mut self, id: usize, input: &Input) -> bool {
        if id >= GAMES.len() {
            return false;
        }
        let mut c = Challenge::new(id);
        c.target = self.projected(Role::King).or(self.field_target);
        c.queen_target = self.projected(Role::Queen);
        c.owner_last = input.owner.cell();
        c.board_serial = self.board.map(|b| b.serial);
        self.challenge = Some(c);
        true
    }
    fn protects(&self, a: usize, b: usize, input: &Input) -> bool {
        self.pieces
            .get(&b)
            .is_some_and(|p| self.threatens(a, p.cell, input))
    }
    fn knight_covers(&self, n: usize, c: Cell, input: &Input) -> bool {
        DIRS.iter()
            .any(|(x, y)| self.threatens(n, Cell::new(c.x + x, c.y + y), input))
    }
    fn kind_ids(&self, kind: Kind) -> Vec<usize> {
        self.pieces
            .values()
            .filter(|p| p.kind == kind && !p.yielding)
            .map(|p| p.id)
            .collect()
    }
    // Events originate only from committed movement, actual hits, enemy captures,
    // validated traps, promotion and observed owner motion. There is no lab-only shortcut.
    pub fn challenge_event(
        &mut self,
        input: &Input,
        event: usize,
        piece: usize,
        target: Option<usize>,
        out: &mut Vec<Cmd>,
    ) {
        let Some(mut c) = self.challenge.take() else {
            return;
        };
        if c.done || c.failed {
            self.challenge = Some(c);
            return;
        }
        let key = (input.tick, event, piece, target);
        if c.last_event == Some(key) {
            self.challenge = Some(c);
            return;
        }
        c.last_event = Some(key);
        let king = c.target.and_then(|id| self.visible(input, id));
        let kc = king.map(Fighter::cell);
        let kid = king.map(|f| f.id);
        let qc = c
            .queen_target
            .and_then(|id| self.visible(input, id))
            .map(Fighter::cell);
        let kind = self.pieces.get(&piece).map(|p| p.kind);
        let check = kc.is_some_and(|k| self.threatens(piece, k, input));
        let qs = self.kind_ids(Kind::Queen);
        let rs = self.kind_ids(Kind::Rook);
        let bs = self.kind_ids(Kind::Bishop);
        let ns = self.kind_ids(Kind::Knight);
        let ps = self.kind_ids(Kind::Pawn);
        let a = c.bindings[0];
        let b = c.bindings[1];
        let d = c.bindings[2];
        let mut advance = false;
        let mut expose = None;
        match (c.id, c.progress, event) {
            (0, 0, 0) if kind == Some(Kind::Queen) && check => {
                if let Some(r) = rs.iter().copied().find(|r| self.protects(*r, piece, input)) {
                    c.bindings[0] = Some(piece);
                    c.bindings[1] = Some(r);
                    expose = Some(piece);
                    advance = true;
                }
            }
            (0, 1, 2) if Some(piece) == a => advance = true,
            (0, 2, 1) if Some(piece) == b && target == c.target && check => advance = true,
            (1, 0, 1) if kind == Some(Kind::Rook) && target == c.queen_target => {
                if let Some(n) = ns.iter().copied().find(|n| {
                    self.protects(piece, *n, input)
                        && qc.is_some_and(|q| self.knight_covers(*n, q, input))
                }) {
                    c.bindings[0] = Some(piece);
                    c.bindings[1] = Some(n);
                    advance = true;
                }
            }
            (1, 1, 1)
                if Some(piece) == b
                    && target == c.queen_target
                    && a.is_some_and(|r| self.protects(r, piece, input)) =>
            {
                c.king_cell = qc;
                advance = true;
            }
            (1, 2, 1)
                if Some(piece) == a
                    && target == c.queen_target
                    && qc != c.king_cell
                    && b.is_some_and(|n| {
                        self.protects(piece, n, input)
                            && qc.is_some_and(|q| self.knight_covers(n, q, input))
                    }) =>
            {
                advance = true
            }
            (2, 0, 0) => {
                if let Some((bi, r)) = bs.iter().find_map(|bi| {
                    rs.iter()
                        .find(|r| self.protects(*bi, **r, input))
                        .map(|r| (*bi, *r))
                }) {
                    if let Some(q) = qs.first() {
                        c.bindings[0] = Some(bi);
                        c.bindings[1] = Some(r);
                        c.bindings[2] = Some(*q);
                        expose = Some(*q);
                        advance = true;
                    }
                }
            }
            (2, 1, 2) if Some(piece) == d => advance = true,
            (2, 2, 3)
                if a.zip(b).is_some_and(|(bi, r)| {
                    self.protects(bi, r, input) && kc.is_some_and(|k| self.threatens(r, k, input))
                }) =>
            {
                advance = true
            }
            (3, 0, 0) if kind == Some(Kind::Bishop) && check => {
                if let Some(q) = qs.iter().copied().find(|q| {
                    input.fighters.iter().any(|f| {
                        f.team != input.owner.team
                            && f.visible
                            && f.hp > 0
                            && f.cell().distance(self.pieces[q].cell) <= 1
                    })
                }) {
                    c.bindings[0] = Some(piece);
                    c.bindings[1] = Some(q);
                    expose = Some(q);
                    advance = true;
                }
            }
            (3, 1, 2) if Some(piece) == b => advance = true,
            (3, 2, 4)
                if kind == Some(Kind::Knight)
                    && target.is_some()
                    && target != c.target
                    && bs.iter().any(|bi| {
                        Some(*bi) != a && kc.is_some_and(|k| self.threatens(*bi, k, input))
                    }) =>
            {
                advance = true
            }
            (4, 0, 0) => {
                if let Some(k) = kc {
                    let knights: Vec<_> = ns
                        .iter()
                        .copied()
                        .filter(|n| self.knight_covers(*n, k, input))
                        .collect();
                    if knights.len() >= 2 {
                        if let Some(q) = qs.first() {
                            c.bindings[0] = Some(knights[0]);
                            c.bindings[1] = Some(knights[1]);
                            c.bindings[2] = Some(*q);
                            expose = Some(*q);
                            advance = true;
                        }
                    }
                }
            }
            (4, 1, 2) if Some(piece) == d => advance = true,
            (4, 2, 3)
                if a.zip(b).is_some_and(|(na, nb)| {
                    kc.is_some_and(|k| {
                        self.knight_covers(na, k, input)
                            && self.knight_covers(nb, k, input)
                            && bs.iter().any(|bi| self.threatens(*bi, k, input))
                    })
                }) =>
            {
                advance = true
            }
            (5, 0, 1) if kind == Some(Kind::Rook) => {
                c.bindings[0] = Some(piece);
                expose = Some(piece);
                advance = true;
            }
            (5, 1, 2) if Some(piece) == a => advance = true,
            (5, 2, 1)
                if kind == Some(Kind::Rook) && Some(piece) != a && target == c.target && check =>
            {
                c.bindings[1] = Some(piece);
                c.king_cell = kc;
                advance = true;
            }
            (5, 3, 1)
                if kind == Some(Kind::Queen)
                    && target == c.target
                    && check
                    && kc != c.king_cell =>
            {
                advance = true
            }
            (6, 0, 1) if kind == Some(Kind::Bishop) && target == kid && check => {
                c.bindings[0] = Some(piece);
                expose = Some(piece);
                advance = true;
            }
            (6, 1, 2) if Some(piece) == a => advance = true,
            (6, 2, 1) if kind == Some(Kind::Queen) && target == kid && check => {
                c.bindings[1] = Some(piece);
                advance = true;
            }
            (6, 3, 1) if kind == Some(Kind::Bishop) && Some(piece) != a && target == kid => {
                c.bindings[2] = Some(piece);
                expose = Some(piece);
                advance = true;
            }
            (6, 4, 2) if Some(piece) == d => advance = true,
            (6, 5, 1) if kind == Some(Kind::Rook) && target == kid && check => advance = true,
            (7, 0, 1)
                if kind == Some(Kind::Rook)
                    && target.is_some_and(|id| {
                        self.projections
                            .iter()
                            .any(|p| p.id == id && matches!(p.role, Role::Bishop | Role::Knight))
                    }) =>
            {
                if let Some(q) = qs.first() {
                    c.bindings[0] = Some(piece);
                    c.bindings[1] = Some(*q);
                    expose = Some(*q);
                    advance = true;
                }
            }
            (7, 1, 2) if Some(piece) == b => advance = true,
            (7, 2, 1) if kind == Some(Kind::Rook) && Some(piece) != a && target == kid && check => {
                c.bindings[2] = Some(piece);
                c.king_cell = self.pieces.get(&piece).map(|p| p.cell);
                advance = true;
            }
            (7, 3, 1)
                if kind == Some(Kind::Bishop)
                    && target == kid
                    && check
                    && self.pieces.get(&piece).map(|p| p.cell) != c.king_cell =>
            {
                advance = true
            }
            (8, 0, 1) if kind == Some(Kind::Bishop) && target == c.queen_target => {
                if let Some(p) = ps.first() {
                    if let Some(q) = qs.first() {
                        c.bindings[0] = Some(piece);
                        c.bindings[1] = Some(*p);
                        c.bindings[2] = Some(*q);
                        c.pawn_start = self.pieces[p].ranks;
                        expose = Some(piece);
                        advance = true;
                    }
                }
            }
            (8, 1, 2) if Some(piece) == a => advance = true,
            (8, 2, 5)
                if Some(piece) == b
                    && self
                        .pieces
                        .get(&piece)
                        .is_some_and(|p| p.ranks >= c.pawn_start + 2)
                    && d.is_some_and(|q| self.protects(q, piece, input)) =>
            {
                advance = true
            }
            (8, 3, 1) if Some(piece) == d && target == kid && check => advance = true,
            (9, 0..=1, 7) => {
                if let Some(k) = kc {
                    let now = input.owner.cell();
                    if now.distance(k) < c.owner_last.distance(k)
                        && qs.iter().any(|q| self.threatens(*q, k, input))
                    {
                        c.bindings[0] = qs.iter().copied().find(|q| self.threatens(*q, k, input));
                        advance = true;
                    }
                }
            }
            (9, 2, 1)
                if Some(piece) == a
                    && target == kid
                    && check
                    && kc.is_some_and(|k| input.owner.cell().distance(k) == 1) =>
            {
                advance = true
            }
            (10, 0, 6)
                if kind == Some(Kind::Rook)
                    && self
                        .pieces
                        .get(&piece)
                        .is_some_and(|p| p.slot.is_none() && p.qualified && p.ranks >= 6) =>
            {
                c.bindings[0] = Some(piece);
                advance = true;
            }
            (10, 1, 1)
                if Some(piece) == a
                    && target == kid
                    && check
                    && self
                        .pieces
                        .get(&piece)
                        .is_some_and(|p| p.cell.distance(input.owner.cell()) == 1) =>
            {
                advance = true
            }
            (11, 0..=1, 7) => {
                let now = input.owner.cell();
                if (now.x - c.owner_last.x).abs() == 1 && (now.y - c.owner_last.y).abs() == 1 {
                    if let Some(p) = b.or_else(|| {
                        ps.iter().copied().find(|p| {
                            now.distance(self.pieces[p].cell)
                                < c.owner_last.distance(self.pieces[p].cell)
                        })
                    }) {
                        if kc.is_some_and(|k| now.distance(k) < c.owner_last.distance(k))
                            && self.pieces.get(&p).is_some_and(|p| {
                                now.distance(p.cell) < c.owner_last.distance(p.cell)
                            })
                        {
                            c.bindings[1] = Some(p);
                            advance = true;
                        }
                    }
                }
            }
            (11, 2, 5) if Some(piece) == b => advance = true,
            (11, 3, 1) if Some(piece) == b && target == kid => advance = true,
            _ => {}
        }
        if event == 7 {
            c.owner_last = input.owner.cell();
        }
        if let Some(id) = expose {
            if let Some(p) = self.pieces.get_mut(&id) {
                p.exposed = true;
            }
        }
        if advance {
            c.started.get_or_insert(input.tick);
            c.progress += 1;
            if c.progress >= GAMES[c.id].stages {
                c.done = true;
                c.finished = Some(input.tick);
                self.repeat_at[c.id] = input.tick + 3600;
                if input.tick >= self.reward_at
                    && self.reward.as_ref().is_none_or(|r| input.tick >= r.until)
                {
                    self.award(input, &c, out);
                }
            }
        }
        self.challenge = Some(c);
    }
    fn award(&mut self, input: &Input, c: &Challenge, out: &mut Vec<Cmd>) {
        let id = c.id;
        let charges = match id {
            0 | 6 | 11 => 2,
            3 => 3,
            7 => 1,
            _ => 4_294_967_295,
        };
        self.reward = Some(Reward {
            id,
            until: input.tick + 300,
            charges,
            piece: c.bindings[0],
        });
        self.reward_at = input.tick + REWARD_CD;
        self.stats.rewards += 1;
        match id {
            1 => out.push(Cmd::Shield {
                target: None,
                amount: 100 + input.owner.ap * 30 / 100,
                ticks: 300,
            }),
            4 => {
                for p in self.pieces.keys() {
                    out.push(Cmd::Buff {
                        target: Some(*p),
                        kind: 4,
                        ticks: 300,
                    });
                }
            }
            5 | 9 => out.push(Cmd::Buff {
                target: None,
                kind: id,
                ticks: 300,
            }),
            10 => {
                if let Some(p) = c.bindings[0].and_then(|id| self.pieces.get(&id)) {
                    out.push(Cmd::Shield {
                        target: Some(p.id),
                        amount: p.max_hp / 4,
                        ticks: 300,
                    });
                }
            }
            _ => {}
        }
        self.label(8, input, out);
        self.fx(
            input.tick,
            &format!("game{id}"),
            input.owner.x,
            input.owner.y,
            90,
            false,
            out,
        );
    }
    fn consume_reward(&mut self, input: &Input, kind: Kind, out: &mut Vec<Cmd>) -> i64 {
        let mut shield = false;
        let mut factor = 100;
        if let Some(r) = self
            .reward
            .as_mut()
            .filter(|r| input.tick < r.until && r.charges > 0)
        {
            let consumes = match r.id {
                0 if kind == Kind::Rook => {
                    factor = 125;
                    true
                }
                3 if matches!(kind, Kind::Knight | Kind::Bishop) => {
                    factor = 120;
                    true
                }
                6 => {
                    shield = true;
                    true
                }
                7 if matches!(kind, Kind::Rook | Kind::Bishop | Kind::Queen) => {
                    factor = 125;
                    true
                }
                8 if kind == Kind::Pawn => {
                    factor = 150;
                    false
                }
                11 if kind == Kind::Pawn => true,
                _ => false,
            };
            if consumes {
                r.charges -= 1;
            }
        }
        if shield {
            out.push(Cmd::Shield {
                target: None,
                amount: 40 + input.owner.ap / 10,
                ticks: 300,
            });
        }
        factor
    }
    pub fn capture(&mut self, input: &Input, capture: Capture, out: &mut Vec<Cmd>) {
        let Some(p) = self.pieces.get(&capture.piece) else {
            return;
        };
        let enemy = capture.combat
            && capture.by.is_none_or(|id| {
                input
                    .fighters
                    .iter()
                    .any(|f| f.id == id && f.team != input.owner.team)
            });
        let exposed = p.exposed;
        let before = self.challenge.as_ref().map(|c| c.progress);
        if enemy && exposed {
            self.challenge_event(input, 2, capture.piece, capture.by, out);
            self.label(6, input, out);
        }
        if let Some(c) = self.challenge.as_mut() {
            if !c.done
                && !c.failed
                && c.bindings.contains(&Some(capture.piece))
                && before == Some(c.progress)
            {
                c.failed = true;
                c.failure = if enemy { 4 } else { 5 };
                c.finished = Some(input.tick);
            }
        }
        self.pieces.remove(&capture.piece);
        for slot in &mut self.slots {
            if *slot == Some(capture.piece) {
                *slot = None;
            }
        }
        self.stats.captures += usize::from(enemy);
        if self.pending.as_ref().is_some_and(|p|matches!(p.action,Action::Move{piece,..}|Action::Castle{piece}if piece==capture.piece)){self.pending=None;}
        self.reindex();
        out.push(Cmd::Despawn {
            piece: capture.piece,
        });
    }
    fn trap(&self, target: Cell, input: &Input) -> Option<BTreeSet<usize>> {
        let mut contributors = BTreeSet::new();
        let mut checked = false;
        for (dx, dy) in std::iter::once((0, 0)).chain(DIRS) {
            let c = Cell::new(target.x + dx, target.y + dy);
            let (x, y) = c.world();
            if (dx != 0 || dy != 0) && (input.wall)(x, y) {
                continue;
            }
            let mut covered = false;
            for id in self.pieces.keys() {
                if self.threatens(*id, c, input) {
                    contributors.insert(*id);
                    covered = true;
                }
            }
            if dx == 0 && dy == 0 {
                checked = covered;
            } else if !covered {
                return None;
            }
        }
        (checked && contributors.len() >= 2).then_some(contributors)
    }
    fn checkmate(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        if !self.has_move {
            return;
        }
        let board_king = self
            .projected(Role::King)
            .and_then(|id| self.visible(input, id))
            .filter(|k| {
                self.board.is_some_and(|b| b.inside(k.x, k.y)) && input.tick >= self.mate_at
            });
        let candidates = [
            board_king.map(|k| (k, true)),
            self.field_target
                .and_then(|id| self.visible(input, id))
                .filter(|k| {
                    self.board.is_none_or(|b| !b.inside(k.x, k.y))
                        && input.tick >= self.field_mate_at
                })
                .map(|k| (k, false)),
        ];
        let candidate = candidates.into_iter().flatten().find_map(|(k, board)| {
            if input.tick < k.mate_immune_until
                || input.tick < *self.immune.get(&k.id).unwrap_or(&0)
            {
                return None;
            }
            self.trap(k.cell(), input).map(|pieces| (k, board, pieces))
        });
        let Some((king, board, contributors)) = candidate else {
            self.mate_warning = None;
            self.mate_target = None;
            self.mate_cell = None;
            return;
        };
        let cell = king.cell();
        if self.mate_target != Some(king.id)
            || self.mate_cell != Some(cell)
            || self.mate_board != board
            || self.mate_warning.is_none()
        {
            self.mate_target = Some(king.id);
            self.mate_cell = Some(cell);
            self.mate_warning = Some(input.tick);
            self.mate_board = board;
            self.label(2, input, out);
        }
        if input.tick < self.mate_warning.unwrap() + MATE_WARN {
            return;
        }
        let targets: Vec<_> = input
            .fighters
            .iter()
            .filter(|f| {
                f.team != input.owner.team
                    && f.hp > 0
                    && f.targetable
                    && input.tick >= f.mate_immune_until
                    && input.tick >= *self.immune.get(&f.id).unwrap_or(&0)
                    && if board {
                        self.board.is_some_and(|b| b.inside(f.x, f.y))
                    } else {
                        f.id == king.id
                    }
            })
            .map(|f| f.id)
            .collect();
        if targets.is_empty() {
            return;
        }
        for target in targets {
            out.push(Cmd::Stun {
                target,
                ticks: if board { 75 } else { 45 },
            });
            out.push(Cmd::Immune { target, ticks: 720 });
            self.immune.insert(target, input.tick + 720);
        }
        if board {
            self.mate_at = input.tick + MATE_CD;
        } else {
            self.field_mate_at = input.tick + 480;
            self.stats.field_mates += 1;
        }
        self.stats.mates += 1;
        self.glow = contributors;
        self.glow_until = input.tick + 75;
        self.mate_warning = None;
        self.label(3, input, out);
        self.challenge_event(input, 3, 4_294_967_295, Some(king.id), out);
        let (x, y) = cell.world();
        self.fx(input.tick, "checkmate", x, y, 90, false, out);
    }
}
