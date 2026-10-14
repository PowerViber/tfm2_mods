//! Magnus's deterministic rules, shared by the native adapter and the browser WASM build.
//! No host handles, clocks, files or global combat state enter this module.
use crate::magnus_data::{recovery_tag, Pattern, GAMES};
use std::collections::BTreeSet;
use std::fmt::Write;

pub const TILE: i64 = 24_000;
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
pub const RESET: usize = 960;
pub const REWARD_CD: usize = 1200;
pub const MATE_CD: usize = 1200;
pub const MATE_WARN: usize = 24;
pub const CASTLE_CD: usize = 720;
pub const WINDOW: usize = 480;
const DIRS: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
];

#[derive(Clone, Debug)]
struct Geometry {
    board: Board,
    blocked: u64,
    edges: [u64; 8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Rook,
    Bishop,
    Knight,
    Queen,
}
impl Kind {
    pub fn code(self) -> i64 {
        match self {
            Self::Rook => 0,
            Self::Bishop => 1,
            Self::Knight => 2,
            Self::Queen => 3,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Rook => "rook",
            Self::Bishop => "bishop",
            Self::Knight => "knight",
            Self::Queen => "queen",
        }
    }
    pub fn hp_pct(self) -> i64 {
        [30, 20, 25, 35][self.code() as usize]
    }
    pub fn recovery(self) -> usize {
        [600, 480, 480, 840][self.code() as usize]
    }
    pub fn damage(self, ap: i64) -> i64 {
        let k = self.code() as usize;
        [35, 30, 45, 65][k] + ap.max(0) * [35, 40, 35, 55][k] / 100
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
    pub fn code(self) -> i64 {
        match self {
            Self::King => 0,
            Self::Queen => 1,
            Self::Rook => 2,
            Self::Bishop => 3,
            Self::Knight => 4,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub x: i8,
    pub y: i8,
}
impl Cell {
    pub fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
    pub fn valid(self) -> bool {
        (0..8).contains(&self.x) && (0..8).contains(&self.y)
    }
    pub fn index(self) -> usize {
        self.y as usize * 8 + self.x as usize
    }
    pub fn bit(self) -> u64 {
        if self.valid() {
            1u64 << self.index()
        } else {
            0
        }
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
}
#[derive(Clone, Debug)]
pub struct Piece {
    pub kind: Kind,
    pub cell: Cell,
    pub home: Cell,
    pub hp: i64,
    pub max_hp: i64,
    pub ready_at: usize,
    pub exposed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub id: usize,
    pub role: Role,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub x: i64,
    pub y: i64,
    pub heading: u8,
    pub serial: usize,
}
impl Board {
    fn rotate(&self, x: i64, y: i64) -> (i64, i64) {
        match self.heading % 4 {
            0 => (x, y),
            1 => (y, -x),
            2 => (-x, -y),
            _ => (-y, x),
        }
    }
    pub fn world(&self, c: Cell) -> (i64, i64) {
        let (x, y) = self.rotate((c.x as i64 - 4) * TILE, c.y as i64 * TILE);
        (self.x + x, self.y + y)
    }
    pub fn cell(&self, x: i64, y: i64) -> Cell {
        let (x, y) = (x - self.x, y - self.y);
        let (x, y) = match self.heading % 4 {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            _ => (y, -x),
        };
        Cell::new(
            ((x + TILE / 2).div_euclid(TILE) + 4).clamp(-120, 120) as i8,
            ((y + TILE / 2).div_euclid(TILE)).clamp(-120, 120) as i8,
        )
    }
    pub fn inside(&self, x: i64, y: i64) -> bool {
        self.cell(x, y).valid()
    }
    pub fn control(&self, x: i64, y: i64) -> bool {
        let c = self.cell(x, y);
        (-1..9).contains(&c.x) && (-1..9).contains(&c.y)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Deploy { heading: u8 },
    Move { piece: usize, to: Cell },
    Castle { piece: usize },
    Dismiss,
}
#[derive(Clone, Copy, Debug)]
pub struct Capture {
    pub piece: usize,
    pub by: Option<usize>,
    pub combat: bool,
}
pub struct Input<'a> {
    pub tick: usize,
    pub owner: &'a Fighter,
    pub fighters: &'a [Fighter],
    pub wall: &'a dyn Fn(i64, i64) -> bool,
    pub captures: &'a [Capture],
    pub piece_hp: [Option<i64>; 7],
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
        piece: usize,
        target: usize,
        amount: i64,
    },
    Stun {
        target: usize,
        ticks: usize,
    },
    Shield {
        amount: i64,
        ticks: usize,
    },
    OwnerPos {
        x: i64,
        y: i64,
    },
    Fx {
        tag: &'static str,
        x: i64,
        y: i64,
        ticks: usize,
    },
    Label {
        code: usize,
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
    pub before: [u64; 7],
}
#[derive(Clone, Debug)]
pub struct Challenge {
    pub id: usize,
    pub progress: usize,
    pub started: Option<usize>,
    pub done: bool,
    pub failed: bool,
    pub failure: usize,
    pub a: Option<usize>,
    pub b: Option<usize>,
    pub target: Option<usize>,
    pub king_cell: Option<Cell>,
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
            a: None,
            b: None,
            target: None,
            king_cell: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub commands: [usize; 4],
    pub captures: usize,
    pub mates: usize,
    pub rewards: usize,
    pub resets: usize,
    pub damage: i64,
    pub casts: usize,
    pub cancelled: usize,
}
#[derive(Clone, Debug)]
pub struct Core {
    pub owner: usize,
    pub rank: usize,
    pub board: Option<Board>,
    pub pieces: [Piece; 7],
    pub projections: Vec<Projection>,
    pub pending: Option<Pending>,
    pub challenge: Option<Challenge>,
    pub previous_challenge: Option<usize>,
    pub reset_at: usize,
    pub castle_at: usize,
    pub mate_at: usize,
    pub reward_at: usize,
    pub brilliant_until: usize,
    pub mate_warning: Option<usize>,
    pub mate_cell: Option<Cell>,
    pub mate_used: bool,
    pub has_move: bool,
    pub stats: Stats,
    pub last_label: usize,
    pub label_until: usize,
    pub last_tick: Option<usize>,
    pub blocked: u64,
    pub external_blockers: u64,
    pub owner_cell: Cell,
    seen: Vec<(usize, i64, i64, usize, i64, i64)>,
    pub coverage: [u64; 7],
    pub glow_mask: u8,
    pub glow_until: usize,
    warn_key: (usize, u64),
    warn_drawn: u64,
    geometry: Option<Geometry>,
    rng: u64,
    serial: usize,
    last_fx: usize,
    tokens: usize,
    token_tick: usize,
    fx_tick: usize,
    fx_count: usize,
}
impl Core {
    pub fn new(seed: u64, owner: usize, rank: usize) -> Self {
        Self {
            owner,
            rank: rank.min(7),
            board: None,
            pieces: std::array::from_fn(|i| Piece {
                kind: PIECE_KINDS[i],
                cell: Cell::new(i as i8, 0),
                home: Cell::new(i as i8, 0),
                hp: 0,
                max_hp: 1,
                ready_at: 0,
                exposed: false,
            }),
            projections: vec![],
            pending: None,
            challenge: None,
            previous_challenge: None,
            reset_at: 0,
            castle_at: 0,
            mate_at: 0,
            reward_at: 0,
            brilliant_until: 0,
            mate_warning: None,
            mate_cell: None,
            mate_used: false,
            has_move: false,
            stats: Stats::default(),
            last_label: 0,
            label_until: 0,
            last_tick: None,
            blocked: 0,
            external_blockers: 0,
            owner_cell: Cell::new(4, 0),
            seen: vec![],
            coverage: [0; 7],
            glow_mask: 0,
            glow_until: 0,
            warn_key: (0, 0),
            warn_drawn: 0,
            geometry: None,
            rng: (seed ^ (owner as u64).wrapping_mul(0x9e3779b97f4a7c15)).max(1),
            serial: 0,
            last_fx: 0,
            tokens: 12,
            token_tick: 0,
            fx_tick: 0,
            fx_count: 0,
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
        tag: &'static str,
        x: i64,
        y: i64,
        ticks: usize,
        ambient: bool,
        out: &mut Vec<Cmd>,
    ) -> bool {
        let refill = tick.saturating_sub(self.token_tick) / 2;
        self.tokens = (self.tokens + refill).min(12);
        self.token_tick += refill * 2;
        if self.fx_tick != tick {
            self.fx_tick = tick;
            self.fx_count = 0;
        }
        if self.tokens == 0 || self.fx_count >= 6 || (ambient && self.tokens <= 4) {
            return false;
        }
        self.tokens -= 1;
        self.fx_count += 1;
        out.push(Cmd::Fx { tag, x, y, ticks });
        true
    }
    fn label(&mut self, code: usize, tick: usize, out: &mut Vec<Cmd>) {
        self.last_label = code;
        self.label_until = tick + 90;
        out.push(Cmd::Label { code });
        let (x, y) = self.board.map_or((0, 0), |b| b.world(self.owner_cell));
        self.fx(
            tick,
            [
                "label0", "label1", "label2", "label3", "label4", "label5", "label6", "label7",
                "label8", "label9", "label10", "label11",
            ][code.min(11)],
            x,
            y,
            90,
            false,
            out,
        );
    }
    pub fn dismiss(&mut self, out: &mut Vec<Cmd>) {
        for i in 0..7 {
            out.push(Cmd::Despawn { piece: i });
        }
        self.board = None;
        self.pending = None;
        self.projections.clear();
        self.challenge = None;
        self.mate_warning = None;
        self.mate_cell = None;
        self.blocked = 0;
        self.external_blockers = 0;
        self.coverage = [0; 7];
        self.glow_mask = 0;
        self.warn_key = (0, 0);
        self.warn_drawn = 0;
    }
    pub fn on_dead(&mut self) -> Vec<Cmd> {
        let mut out = vec![];
        self.dismiss(&mut out);
        out
    }
    pub fn duration(&self, kind: Kind, tick: usize) -> usize {
        let base = if kind == Kind::Queen { 45 } else { 30 };
        if tick < self.brilliant_until {
            base * 4 / 5
        } else {
            base
        }
    }
    fn occupant(&self, c: Cell, except: usize, me: Cell) -> bool {
        c == me
            || self.external_blockers & c.bit() != 0
            || self
                .pieces
                .iter()
                .enumerate()
                .any(|(i, p)| i != except && p.hp > 0 && p.cell == c)
    }
    fn terrain(&self, c: Cell, wall: &dyn Fn(i64, i64) -> bool) -> bool {
        let Some(b) = self.board else { return true };
        if c.valid() {
            if let Some(g) = &self.geometry {
                if g.board == b {
                    return g.blocked & c.bit() != 0;
                }
            }
        }
        let (x, y) = b.world(c);
        wall(x, y)
    }
    fn edge_wall(&self, from: Cell, to: Cell, wall: &dyn Fn(i64, i64) -> bool) -> bool {
        let Some(b) = self.board else { return true };
        let d = (to.x - from.x, to.y - from.y);
        if from.valid() && to.valid() {
            if let Some(g) = &self.geometry {
                if g.board == b {
                    if let Some(i) = DIRS.iter().position(|&a| a == d) {
                        return g.edges[i] & from.bit() != 0;
                    }
                }
            }
        }
        let a = b.world(from);
        let z = b.world(to);
        (1..=12).any(|k| wall(a.0 + (z.0 - a.0) * k / 12, a.1 + (z.1 - a.1) * k / 12))
    }
    fn cache_geometry(&mut self, wall: &dyn Fn(i64, i64) -> bool) {
        let Some(b) = self.board else { return };
        let mut g = Geometry {
            board: b,
            blocked: 0,
            edges: [0; 8],
        };
        for n in 0..64 {
            let c = Cell::new(n % 8, n / 8);
            let a = b.world(c);
            if wall(a.0, a.1) {
                g.blocked |= c.bit();
            }
            for (i, (dx, dy)) in DIRS.iter().enumerate() {
                let to = Cell::new(c.x + dx, c.y + dy);
                if !to.valid() {
                    continue;
                }
                let z = b.world(to);
                if (1..=12).any(|k| wall(a.0 + (z.0 - a.0) * k / 12, a.1 + (z.1 - a.1) * k / 12)) {
                    g.edges[i] |= c.bit();
                }
            }
        }
        self.geometry = Some(g);
    }
    pub fn legal(&self, piece: usize, to: Cell, me: Cell, wall: &dyn Fn(i64, i64) -> bool) -> bool {
        if self.board.is_none() {
            return false;
        }
        let Some(p) = self.pieces.get(piece) else {
            return false;
        };
        if p.hp <= 0 || !to.valid() || to == p.cell || self.occupant(to, piece, me) {
            return false;
        }
        if self.terrain(to, wall) {
            return false;
        }
        let (dx, dy) = (to.x - p.cell.x, to.y - p.cell.y);
        if p.kind == Kind::Knight {
            return (dx.abs() == 1 && dy.abs() == 2) || (dx.abs() == 2 && dy.abs() == 1);
        }
        let straight = dx == 0 || dy == 0;
        let diagonal = dx.abs() == dy.abs();
        if !(match p.kind {
            Kind::Rook => straight,
            Kind::Bishop => diagonal,
            Kind::Queen => straight || diagonal,
            Kind::Knight => false,
        }) {
            return false;
        }
        let mut c = p.cell;
        let (sx, sy) = (dx.signum(), dy.signum());
        while c != to {
            let old = c;
            c = Cell::new(c.x + sx, c.y + sy);
            if self.occupant(c, piece, me) || self.terrain(c, wall) {
                return false;
            }
            if self.edge_wall(old, c, wall) {
                return false;
            }
        }
        true
    }
    /// Geometric attack coverage includes defended friendly squares, but stops behind them.
    pub fn attacks(&self, piece: usize, wall: &dyn Fn(i64, i64) -> bool) -> u64 {
        if self.board.is_none() {
            return 0;
        }
        let p = &self.pieces[piece];
        if p.hp <= 0 {
            return 0;
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|a| a.committed && matches!(a.action,Action::Move{piece:i,..}if i==piece))
        {
            return 0;
        }
        if p.kind == Kind::Knight {
            return [
                (1, 2),
                (2, 1),
                (-1, 2),
                (-2, 1),
                (1, -2),
                (2, -1),
                (-1, -2),
                (-2, -1),
            ]
            .iter()
            .fold(0, |mask, (x, y)| {
                let c = Cell::new(p.cell.x + x, p.cell.y + y);
                if c.valid() && !self.terrain(c, wall) {
                    mask | c.bit()
                } else {
                    mask
                }
            });
        }
        let dirs: [(i8, i8); 8] = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ];
        let mut mask = 0;
        for (dx, dy) in dirs {
            if (p.kind == Kind::Rook && dx != 0 && dy != 0)
                || (p.kind == Kind::Bishop && (dx == 0 || dy == 0))
            {
                continue;
            }
            let mut c = p.cell;
            loop {
                let old = c;
                c = Cell::new(c.x + dx, c.y + dy);
                if !c.valid() {
                    break;
                }
                if self.terrain(c, wall) || self.edge_wall(old, c, wall) {
                    break;
                }
                mask |= c.bit();
                if c == self.owner_cell
                    || self.external_blockers & c.bit() != 0
                    || self
                        .pieces
                        .iter()
                        .enumerate()
                        .any(|(i, q)| i != piece && q.hp > 0 && q.cell == c)
                {
                    break;
                }
            }
        }
        mask
    }
    pub fn masks(&self, wall: &dyn Fn(i64, i64) -> bool) -> [u64; 7] {
        std::array::from_fn(|i| self.attacks(i, wall))
    }
    fn enemy_role(f: &Fighter) -> Role {
        if f.tank {
            Role::Rook
        } else if f.ranged {
            Role::Bishop
        } else {
            Role::Knight
        }
    }
    fn project(&mut self, input: &Input, initial: bool) {
        let Some(b) = self.board else { return };
        let enemies: Vec<_> = input
            .fighters
            .iter()
            .filter(|f| {
                f.team != input.owner.team
                    && f.champion
                    && f.hp > 0
                    && f.visible
                    && f.targetable
                    && b.inside(f.x, f.y)
            })
            .collect();
        if initial {
            self.projections.clear();
            let king = enemies
                .iter()
                .min_by_key(|f| {
                    let dx = f.x - input.owner.x;
                    let dy = f.y - input.owner.y;
                    ((dx as i128).pow(2) + (dy as i128).pow(2), f.id)
                })
                .map(|f| f.id);
            let queen = enemies
                .iter()
                .filter(|f| Some(f.id) != king)
                .max_by_key(|f| (f.attack + f.ap, usize::MAX - f.id))
                .map(|f| f.id);
            for f in &enemies {
                self.projections.push(Projection {
                    id: f.id,
                    role: if Some(f.id) == king {
                        Role::King
                    } else if Some(f.id) == queen {
                        Role::Queen
                    } else {
                        Self::enemy_role(f)
                    },
                })
            }
        } else {
            for f in enemies {
                if !self.projections.iter().any(|p| p.id == f.id) {
                    self.projections.push(Projection {
                        id: f.id,
                        role: Self::enemy_role(f),
                    });
                }
            }
        }
        self.projections.sort_by_key(|p| p.id);
    }
    pub fn projected(&self, role: Role) -> Option<usize> {
        self.projections
            .iter()
            .find(|p| p.role == role)
            .map(|p| p.id)
    }
    fn visible<'a>(&self, input: &'a Input, id: usize) -> Option<&'a Fighter> {
        input.fighters.iter().find(|f| {
            f.id == id
                && f.hp > 0
                && f.visible
                && f.targetable
                && self.board.is_some_and(|b| b.inside(f.x, f.y))
        })
    }
    fn king<'a>(&self, input: &'a Input) -> Option<&'a Fighter> {
        self.projected(Role::King)
            .and_then(|id| self.visible(input, id))
    }
    fn begin(&mut self, input: &Input, action: Action) -> bool {
        if self.pending.is_some() || input.controlled {
            return false;
        }
        let (duration, from) = match action {
            Action::Deploy { heading } => {
                if input.tick < self.reset_at {
                    return false;
                }
                let b = Board {
                    x: input.owner.x,
                    y: input.owner.y,
                    heading: heading % 4,
                    serial: self.serial + 1,
                };
                if (0..8).any(|x| {
                    let p = b.world(Cell::new(x, 0));
                    (input.wall)(p.0, p.1)
                }) {
                    return false;
                }
                (60, Cell::new(4, 0))
            }
            Action::Move { piece, to } => {
                let Some(b) = self.board else { return false };
                if !b.control(input.owner.x, input.owner.y)
                    || !self.legal(piece, to, b.cell(input.owner.x, input.owner.y), input.wall)
                {
                    return false;
                }
                (
                    self.duration(self.pieces[piece].kind, input.tick),
                    self.pieces[piece].cell,
                )
            }
            Action::Castle { piece } => {
                let Some(b) = self.board else { return false };
                let me = b.cell(input.owner.x, input.owner.y);
                let Some(p) = self.pieces.get(piece) else {
                    return false;
                };
                let (dx, dy) = ((p.cell.x - me.x).abs(), (p.cell.y - me.y).abs());
                if input.tick < self.castle_at
                    || p.kind != Kind::Rook
                    || p.hp <= 0
                    || !me.valid()
                    || !(dx == 0 || dy == 0)
                    || dx + dy > 3
                    || dx + dy == 0
                {
                    return false;
                }
                if !self.castle_legal(input, piece, me) {
                    return false;
                }
                let mut c = me;
                let (sx, sy) = ((p.cell.x - me.x).signum(), (p.cell.y - me.y).signum());
                while c != p.cell {
                    c = Cell::new(c.x + sx, c.y + sy);
                    let (x, y) = b.world(c);
                    if (input.wall)(x, y)
                        || self
                            .pieces
                            .iter()
                            .enumerate()
                            .any(|(i, q)| i != piece && q.hp > 0 && q.cell == c)
                    {
                        return false;
                    }
                }
                (self.duration(Kind::Rook, input.tick), me)
            }
            Action::Dismiss => return true,
        };
        let warn = if matches!(action, Action::Deploy { .. }) {
            duration
        } else {
            (duration * 3 + 2) / 5
        };
        let before = self.masks(input.wall);
        self.pending = Some(Pending {
            action,
            start: input.tick,
            commit: input.tick + warn,
            end: input.tick + duration,
            from,
            hit: BTreeSet::new(),
            committed: false,
            before,
        });
        true
    }
    fn castle_legal(&self, input: &Input, piece: usize, from: Cell) -> bool {
        let Some(b) = self.board else { return false };
        let Some(p) = self.pieces.get(piece) else {
            return false;
        };
        let me = b.cell(input.owner.x, input.owner.y);
        let (dx, dy) = ((p.cell.x - me.x).abs(), (p.cell.y - me.y).abs());
        if me != from
            || p.hp <= 0
            || p.kind != Kind::Rook
            || !me.valid()
            || !(dx == 0 || dy == 0)
            || dx + dy == 0
            || dx + dy > 3
        {
            return false;
        }
        let mut c = me;
        let (sx, sy) = ((p.cell.x - me.x).signum(), (p.cell.y - me.y).signum());
        while c != p.cell {
            let old = b.world(c);
            c = Cell::new(c.x + sx, c.y + sy);
            let new = b.world(c);
            if self.external_blockers & c.bit() != 0
                || self
                    .pieces
                    .iter()
                    .enumerate()
                    .any(|(i, q)| i != piece && q.hp > 0 && q.cell == c)
                || (1..=8).any(|k| {
                    (input.wall)(
                        old.0 + (new.0 - old.0) * k / 8,
                        old.1 + (new.1 - old.1) * k / 8,
                    )
                })
            {
                return false;
            }
        }
        true
    }
    fn deploy(&mut self, input: &Input, heading: u8, out: &mut Vec<Cmd>) {
        self.dismiss(out);
        self.serial += 1;
        self.board = Some(Board {
            x: input.owner.x,
            y: input.owner.y,
            heading: heading % 4,
            serial: self.serial,
        });
        self.owner_cell = Cell::new(4, 0);
        self.mate_used = false;
        self.has_move = false;
        self.reset_at = input.tick + RESET;
        self.stats.resets += 1;
        let mut slots = [0i8, 1, 2, 6, 7];
        // Rejection sampling is bounded; all layouts with opposite-colour bishops have equal probability.
        for _ in 0..64 {
            for i in (1..5).rev() {
                let j = self.roll(i + 1);
                slots.swap(i, j)
            }
            if slots[1] % 2 != slots[2] % 2 {
                break;
            }
        }
        if slots[1] % 2 == slots[2] % 2 {
            let j = (0..5)
                .find(|&j| j != 1 && j != 2 && slots[j] % 2 != slots[1] % 2)
                .unwrap();
            slots.swap(2, j)
        }
        let homes = [3, slots[0], slots[1], slots[2], slots[3], slots[4], 5];
        let first = self.stats.resets == 1;
        let b = self.board.unwrap();
        self.external_blockers = input
            .fighters
            .iter()
            .filter(|f| f.team == input.owner.team && f.friendly_chess && f.hp > 0)
            .fold(0, |a, f| a | b.cell(f.x, f.y).bit());
        for i in 0..7 {
            let p = &mut self.pieces[i];
            p.home = Cell::new(homes[i], 0);
            p.cell = p.home;
            p.exposed = false;
            p.max_hp = (input.owner.max_hp * p.kind.hp_pct() / 100).max(1);
            if first || i == 0 || i == 6 {
                p.hp = p.max_hp;
                p.ready_at = input.tick
            } else {
                p.hp = p.hp.min(p.max_hp)
            }
            if p.hp > 0 {
                let (x, y) = b.world(p.cell);
                out.push(Cmd::Spawn {
                    piece: i,
                    x,
                    y,
                    hp: p.hp,
                })
            }
        }
        self.project(input, true);
        let mut compatible = self.compatible();
        if compatible.len() > 1 {
            compatible.retain(|id| Some(*id) != self.previous_challenge)
        }
        if !compatible.is_empty() {
            let id = compatible[self.roll(compatible.len())];
            self.challenge = Some(Challenge::new(id));
            self.previous_challenge = Some(id)
        }
        let a = b.world(Cell::new(0, 0));
        let z = b.world(Cell::new(7, 7));
        self.fx(
            input.tick,
            ["board0", "board1", "board2", "board3"][b.heading as usize],
            (a.0 + z.0) / 2,
            (a.1 + z.1) / 2,
            248,
            false,
            out,
        );
        self.last_fx = input.tick;
        self.label(1, input.tick, out);
    }
    pub fn compatible(&self) -> Vec<usize> {
        if self.projected(Role::King).is_none() {
            return vec![];
        }
        let counts = std::array::from_fn::<_, 4, _>(|kind| {
            self.pieces
                .iter()
                .filter(|p| p.hp > 0 && p.kind.code() as usize == kind)
                .count()
        });
        GAMES
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                g.pieces.iter().enumerate().all(|(i, n)| counts[i] >= *n)
                    && (!g.queen_target || self.projected(Role::Queen).is_some())
            })
            .map(|(i, _)| i)
            .collect()
    }
    fn challenge_event(
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
        if c.failed || c.done {
            self.challenge = Some(c);
            return;
        }
        let Some(b) = self.board else {
            self.challenge = Some(c);
            return;
        };
        let masks = self.masks(input.wall);
        let king = self.king(input);
        let kc = king.map(|k| b.cell(k.x, k.y));
        let kid = king.map(|k| k.id);
        let check = kc.is_some_and(|cell| masks.get(piece).copied().unwrap_or(0) & cell.bit() != 0);
        let alive = |i: usize| self.pieces.get(i).is_some_and(|p| p.hp > 0);
        let protects =
            |i: usize, j: usize| alive(i) && alive(j) && masks[i] & self.pieces[j].cell.bit() != 0;
        let rook_protector = (0..2).find(|&r| protects(r, 6));
        let bishop_rook = (2..4).find_map(|bi| (0..2).find(|&r| protects(bi, r)).map(|r| (bi, r)));
        let queen_cell = self
            .projected(Role::Queen)
            .and_then(|id| self.visible(input, id))
            .map(|f| b.cell(f.x, f.y));
        let guarded_knight = (4..6).find(|&n| {
            protects(piece, n) && queen_cell.is_some_and(|q| masks[n] & neighbors(q) != 0)
        });
        let mut advance = false;
        use Pattern::*;
        match (GAMES[c.id].pattern, c.progress, event) {
            (Birthday, 0, 0) if piece == 6 && check && rook_protector.is_some() => {
                c.a = rook_protector;
                c.target = kid;
                self.pieces[6].exposed = true;
                advance = true
            }
            (Birthday, 1, 2) if piece == 6 => advance = true,
            (Birthday, 2, 1) if Some(piece) == c.a && target == c.target && check => advance = true,
            (Marathon, 0, 1)
                if piece < 2
                    && target == self.projected(Role::Queen)
                    && guarded_knight.is_some() =>
            {
                c.a = Some(piece);
                c.b = guarded_knight;
                c.target = target;
                advance = true
            }
            (Marathon, 1, 1)
                if Some(piece) == c.b
                    && target == c.target
                    && c.a.is_some_and(|r| protects(r, piece)) =>
            {
                advance = true
            }
            (Marathon, 2, 1)
                if Some(piece) == c.a
                    && target == c.target
                    && c.b.is_some_and(|n| {
                        protects(piece, n)
                            && queen_cell.is_some_and(|q| masks[n] & neighbors(q) != 0)
                    }) =>
            {
                advance = true
            }
            (Opera, 0, 0) if bishop_rook.is_some() => {
                let (bi, r) = bishop_rook.unwrap();
                c.a = Some(bi);
                c.b = Some(r);
                self.pieces[6].exposed = true;
                c.target = kid;
                advance = true
            }
            (Opera, 1, 2) if piece == 6 => advance = true,
            (Opera, 2, 3)
                if c.a.zip(c.b).is_some_and(|(bi, r)| {
                    protects(bi, r) && kc.is_some_and(|k| masks[r] & k.bit() != 0)
                }) =>
            {
                advance = true
            }
            (Century, 0, 0)
                if (2..4).contains(&piece)
                    && check
                    && input.fighters.iter().any(|f| {
                        f.team != input.owner.team
                            && f.hp > 0
                            && f.visible
                            && b.cell(f.x, f.y) == self.pieces[6].cell
                    }) =>
            {
                c.a = Some(piece);
                c.target = kid;
                self.pieces[6].exposed = true;
                advance = true
            }
            (Century, 1, 2) if piece == 6 => advance = true,
            (Century, 2, 4)
                if (4..6).contains(&piece)
                    && target != c.target
                    && target.is_some()
                    && (2..4).any(|bi| {
                        Some(bi) != c.a && kc.is_some_and(|k| masks[bi] & k.bit() != 0)
                    }) =>
            {
                advance = true
            }
            (Immortal, 0, 0)
                if kc.is_some_and(|k| {
                    masks[4] & neighbors(k) != 0 && masks[5] & neighbors(k) != 0
                }) =>
            {
                c.target = kid;
                self.pieces[6].exposed = true;
                advance = true
            }
            (Immortal, 1, 2) if piece == 6 => advance = true,
            (Immortal, 2, 3)
                if kc.is_some_and(|k| {
                    (masks[2] | masks[3]) & k.bit() != 0
                        && masks[4] & neighbors(k) != 0
                        && masks[5] & neighbors(k) != 0
                }) =>
            {
                advance = true
            }
            (Hunt, 0, 1) if piece < 2 => {
                c.a = Some(piece);
                c.target = kid;
                self.pieces[piece].exposed = true;
                advance = true
            }
            (Hunt, 1, 2) if Some(piece) == c.a => advance = true,
            (Hunt, 2, 1) if piece < 2 && Some(piece) != c.a && target == c.target && check => {
                c.king_cell = kc;
                advance = true
            }
            (Hunt, 3, 1) if piece == 6 && target == c.target && check && kc != c.king_cell => {
                advance = true
            }
            _ => {}
        }
        if advance {
            if c.started.is_none() {
                c.started = Some(input.tick)
            }
            c.progress += 1;
            let stages = GAMES[c.id].stages;
            if c.progress >= stages {
                c.done = true;
                if input.tick >= self.reward_at && input.tick >= self.brilliant_until {
                    self.brilliant_until = input.tick + 300;
                    self.reward_at = input.tick + REWARD_CD;
                    self.stats.rewards += 1;
                    self.label(8, input.tick, out);
                    let (x, y) = (input.owner.x, input.owner.y);
                    self.fx(input.tick, "brilliant", x, y, 90, false, out);
                    self.fx(
                        input.tick,
                        ["game0", "game1", "game2", "game3", "game4", "game5"][c.id],
                        x,
                        y,
                        90,
                        false,
                        out,
                    );
                }
            }
        }
        self.challenge = Some(c);
    }
    pub fn capture(&mut self, input: &Input, capture: Capture, out: &mut Vec<Cmd>) {
        let i = capture.piece;
        if i >= 7 || self.pieces[i].hp <= 0 {
            return;
        }
        let exposed = self.pieces[i].exposed;
        self.pieces[i].hp = 0;
        self.pieces[i].ready_at = input.tick + self.pieces[i].kind.recovery();
        self.stats.captures += 1;
        out.push(Cmd::Despawn { piece: i });
        if let Some(b) = self.board {
            let (x, y) = b.world(self.pieces[i].home);
            self.fx(
                input.tick,
                recovery_tag(i, self.pieces[i].kind.recovery()),
                x,
                y,
                66,
                true,
                out,
            );
        }
        if self.pending.as_ref().is_some_and(
            |p| matches!(p.action,Action::Move{piece,..}|Action::Castle{piece}if piece==i),
        ) {
            self.pending = None
        }
        let enemy_capture = capture.combat
            && capture.by.is_none_or(|id| {
                input
                    .fighters
                    .iter()
                    .any(|f| f.id == id && f.team != input.owner.team)
            });
        if enemy_capture && exposed {
            self.label(6, input.tick, out);
            self.challenge_event(input, 2, i, capture.by, out)
        }
        if let Some(c) = self.challenge.as_mut() {
            if !c.done && !c.failed {
                let expected = (i == 6 && c.id != 1 && c.progress == 2)
                    || (c.id == 5 && Some(i) == c.a && c.progress >= 2);
                let essential = match c.id {
                    0 => i == 6 || Some(i) == c.a,
                    1 => Some(i) == c.a || Some(i) == c.b,
                    2 => i == 6 || Some(i) == c.a || Some(i) == c.b,
                    3 => i == 6 || (i >= 2 && i < 4 && Some(i) != c.a),
                    4 => i == 6 || i == 4 || i == 5,
                    5 => i == 6 || i < 2,
                    _ => false,
                };
                let mut required = GAMES[c.id].pieces;
                if c.id != 1 && c.progress >= 2 {
                    required[3] = 0;
                }
                if c.id == 5 && c.progress >= 2 {
                    required[0] = 1;
                }
                if c.id == 3 && c.progress >= 1 {
                    required[1] = 1;
                }
                let insufficient = required.iter().enumerate().any(|(kind, n)| {
                    self.pieces
                        .iter()
                        .filter(|p| p.hp > 0 && p.kind.code() as usize == kind)
                        .count()
                        < *n
                });
                if (essential && !expected) || insufficient {
                    c.failed = true;
                    c.failure = if enemy_capture { 4 } else { 5 };
                }
            }
        }
    }
    fn checkmate(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        if !self.has_move || self.mate_used || input.tick < self.mate_at {
            return;
        }
        let Some(b) = self.board else { return };
        let Some(k) = self.king(input) else {
            self.mate_warning = None;
            self.mate_cell = None;
            return;
        };
        let cell = b.cell(k.x, k.y);
        let masks = self.masks(input.wall);
        let all = masks.iter().fold(0, |a, m| a | m);
        let mut escape = false;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let c = Cell::new(cell.x + dx, cell.y + dy);
                let (x, y) = b.world(c);
                if !(input.wall)(x, y) && (!c.valid() || all & c.bit() == 0) {
                    escape = true
                }
            }
        }
        let checking = all & cell.bit() != 0;
        let contributors = masks
            .iter()
            .filter(|m| **m & (cell.bit() | neighbors(cell)) != 0)
            .count();
        if escape || !checking || contributors < 2 {
            if !escape && !checking && self.last_label != 7 {
                self.label(7, input.tick, out)
            }
            self.mate_warning = None;
            self.mate_cell = None;
            return;
        }
        if self.mate_warning.is_none() || self.mate_cell != Some(cell) {
            self.mate_warning = Some(input.tick);
            self.mate_cell = Some(cell);
            self.label(2, input.tick, out);
        }
        if input.tick >= self.mate_warning.unwrap() + MATE_WARN {
            self.mate_used = true;
            self.mate_at = input.tick + MATE_CD;
            self.stats.mates += 1;
            self.mate_warning = None;
            self.glow_mask = masks
                .iter()
                .enumerate()
                .filter(|(_, m)| **m & (cell.bit() | neighbors(cell)) != 0)
                .fold(0, |a, (i, _)| a | (1 << i));
            self.glow_until = input.tick + 75;
            for f in input.fighters.iter().filter(|f| {
                f.team != input.owner.team && f.hp > 0 && f.targetable && b.inside(f.x, f.y)
            }) {
                out.push(Cmd::Stun {
                    target: f.id,
                    ticks: 75,
                })
            }
            self.label(3, input.tick, out);
            let (x, y) = b.world(cell);
            self.fx(input.tick, "checkmate", x, y, 90, false, out);
            self.challenge_event(input, 3, 0, Some(k.id), out);
        }
    }
    fn resolve(&mut self, input: &Input, p: &mut Pending, out: &mut Vec<Cmd>) {
        let Some(b) = self.board else { return };
        let Action::Move { piece, to } = p.action else {
            return;
        };
        let elapsed = input.tick.saturating_sub(p.commit).min(p.end - p.commit);
        let duration = (p.end - p.commit).max(1);
        let (a, z) = (b.world(p.from), b.world(to));
        let x = a.0 + (z.0 - a.0) * elapsed as i64 / duration as i64;
        let y = a.1 + (z.1 - a.1) * elapsed as i64 / duration as i64;
        out.push(Cmd::Move { piece, x, y });
        let at = b.cell(x, y);
        let knight = self.pieces[piece].kind == Kind::Knight;
        for f in input.fighters.iter().filter(|f| {
            f.team != input.owner.team
                && f.hp > 0
                && f.targetable
                && f.visible
                && b.inside(f.x, f.y)
        }) {
            let fc = b.cell(f.x, f.y);
            let hit = if knight {
                input.tick >= p.end && fc == to
            } else {
                // Sweep the visited squares, including tick gaps, rather than only the current interpolated cell.
                let dx = (to.x - p.from.x).signum();
                let dy = (to.y - p.from.y).signum();
                let reached = (at.x - p.from.x).abs().max((at.y - p.from.y).abs());
                (1..=reached).any(|j| Cell::new(p.from.x + dx * j, p.from.y + dy * j) == fc)
            };
            if hit && p.hit.insert(f.id) {
                let amount = self.pieces[piece].kind.damage(input.owner.ap);
                self.stats.damage += amount;
                out.push(Cmd::Damage {
                    piece,
                    target: f.id,
                    amount,
                });
            }
        }
    }
    pub fn step(&mut self, input: &Input) -> Vec<Cmd> {
        if self.last_tick.is_some_and(|t| input.tick <= t) {
            return vec![];
        }
        self.last_tick = Some(input.tick);
        let mut out = vec![];
        if input.owner.hp <= 0 {
            self.dismiss(&mut out);
            return out;
        }
        if input.action == Some(Action::Dismiss) {
            self.dismiss(&mut out);
            return out;
        }
        if let Some(b) = self.board {
            self.owner_cell = b.cell(input.owner.x, input.owner.y);
            self.external_blockers = input
                .fighters
                .iter()
                .filter(|f| f.team == input.owner.team && f.friendly_chess && f.hp > 0)
                .fold(0, |a, f| a | b.cell(f.x, f.y).bit());
        }
        for f in input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.visible && f.hp > 0)
        {
            if let Some(s) = self.seen.iter_mut().find(|s| s.0 == f.id) {
                if input.tick > s.3 {
                    let d = (input.tick - s.3) as i64;
                    s.4 = ((f.x - s.1) / d).clamp(-1800, 1800);
                    s.5 = ((f.y - s.2) / d).clamp(-1800, 1800);
                    s.1 = f.x;
                    s.2 = f.y;
                    s.3 = input.tick;
                }
            } else {
                self.seen.push((f.id, f.x, f.y, input.tick, 0, 0));
            }
        }
        self.seen.retain(|s| input.tick.saturating_sub(s.3) < 600);
        for c in input.captures {
            self.capture(input, *c, &mut out)
        }
        for i in 0..7 {
            if let Some(hp) = input.piece_hp[i] {
                if hp > 0 && self.pieces[i].hp > 0 {
                    self.pieces[i].hp = hp.min(self.pieces[i].max_hp)
                }
            }
        }
        if let Some(b) = self.board {
            self.blocked = 0;
            for y in 0..8 {
                for x in 0..8 {
                    let c = Cell::new(x, y);
                    let (wx, wy) = b.world(c);
                    if (input.wall)(wx, wy) {
                        self.blocked |= c.bit()
                    }
                }
            }
            self.project(input, false);
            if let Some(c) = self.challenge.as_mut() {
                if !c.done && !c.failed {
                    let king_lost = self
                        .projections
                        .iter()
                        .find(|p| p.role == Role::King)
                        .is_some_and(|p| {
                            !input.fighters.iter().any(|f| {
                                f.id == p.id && f.hp > 0 && f.visible && b.inside(f.x, f.y)
                            })
                        });
                    let target_lost = c.target.is_some_and(|id| {
                        !input
                            .fighters
                            .iter()
                            .any(|f| f.id == id && f.hp > 0 && f.visible && b.inside(f.x, f.y))
                    });
                    if king_lost {
                        c.failed = true;
                        c.failure = 2;
                    } else if target_lost {
                        c.failed = true;
                        c.failure = 3;
                    } else if c.started.is_some_and(|t| input.tick > t + WINDOW) {
                        c.failed = true;
                        c.failure = 1;
                    }
                }
            }
            for i in 0..7 {
                if self.pieces[i].hp == 0 && input.tick >= self.pieces[i].ready_at {
                    let home = self.pieces[i].home;
                    let color = (home.x + home.y) % 2;
                    let best = (0..64)
                        .map(|n| Cell::new(n % 8, n / 8))
                        .filter(|c| {
                            let (wx, wy) = b.world(*c);
                            !(input.wall)(wx, wy)
                                && !self.occupant(*c, i, b.cell(input.owner.x, input.owner.y))
                                && (self.pieces[i].kind != Kind::Bishop || (c.x + c.y) % 2 == color)
                        })
                        .min_by_key(|c| ((c.x - home.x).abs() + (c.y - home.y).abs(), c.index()));
                    if let Some(cell) = best {
                        let p = &mut self.pieces[i];
                        p.hp = p.max_hp;
                        p.cell = cell;
                        p.exposed = false;
                        let (x, y) = b.world(cell);
                        out.push(Cmd::Spawn {
                            piece: i,
                            x,
                            y,
                            hp: p.hp,
                        });
                    }
                } else if self.pieces[i].hp == 0
                    && self.pieces[i].ready_at < 1_000_000_000
                    && input.tick.is_multiple_of(60)
                {
                    let (x, y) = b.world(self.pieces[i].home);
                    self.fx(
                        input.tick,
                        recovery_tag(i, self.pieces[i].ready_at - input.tick),
                        x,
                        y,
                        66,
                        true,
                        &mut out,
                    );
                }
            }
            if input.tick >= self.last_fx + 240 {
                let a = b.world(Cell::new(0, 0));
                let z = b.world(Cell::new(7, 7));
                if self.fx(
                    input.tick,
                    ["board0", "board1", "board2", "board3"][b.heading as usize],
                    (a.0 + z.0) / 2,
                    (a.1 + z.1) / 2,
                    248,
                    true,
                    &mut out,
                ) {
                    self.last_fx = input.tick
                }
            }
        }
        if let Some(mut p) = self.pending.take() {
            if !p.committed && input.controlled {
                self.stats.cancelled += 1;
                self.label(9, input.tick, &mut out);
            } else {
                if !p.committed && input.tick >= p.commit {
                    let valid = match p.action {
                        Action::Move { piece, to } => self.board.is_some_and(|b| {
                            b.control(input.owner.x, input.owner.y)
                                && self.legal(
                                    piece,
                                    to,
                                    b.cell(input.owner.x, input.owner.y),
                                    input.wall,
                                )
                        }),
                        Action::Castle { piece } => self.castle_legal(input, piece, p.from),
                        Action::Deploy { heading } => {
                            let b = Board {
                                x: input.owner.x,
                                y: input.owner.y,
                                heading,
                                serial: 0,
                            };
                            (0..8).all(|x| {
                                let p = b.world(Cell::new(x, 0));
                                !(input.wall)(p.0, p.1)
                            })
                        }
                        Action::Dismiss => true,
                    };
                    if !valid {
                        self.stats.cancelled += 1;
                        self.label(9, input.tick, &mut out);
                        return out;
                    } else {
                        p.before = self.masks(input.wall);
                        p.committed = true;
                        match p.action {
                            Action::Deploy { heading } => self.deploy(input, heading, &mut out),
                            Action::Castle { piece } => {
                                let b = self.board.unwrap();
                                let old = b.cell(input.owner.x, input.owner.y);
                                let (x, y) = b.world(self.pieces[piece].cell);
                                self.owner_cell = self.pieces[piece].cell;
                                self.pieces[piece].cell = old;
                                out.push(Cmd::OwnerPos { x, y });
                                out.push(Cmd::Move {
                                    piece,
                                    x: b.world(old).0,
                                    y: b.world(old).1,
                                });
                                out.push(Cmd::Shield {
                                    amount: 80 + input.owner.ap / 5,
                                    ticks: 60,
                                });
                                self.castle_at = input.tick + CASTLE_CD;
                                self.stats.casts += 1;
                                self.has_move = true;
                                self.label(10, input.tick, &mut out);
                            }
                            Action::Move { piece, .. } => {
                                self.stats.commands[self.pieces[piece].kind.code() as usize] += 1;
                            }
                            Action::Dismiss => self.dismiss(&mut out),
                        }
                    }
                }
                if p.committed {
                    self.resolve(input, &mut p, &mut out);
                    if input.tick >= p.end {
                        if let Action::Move { piece, to } = p.action {
                            self.pieces[piece].cell = to;
                            self.has_move = true;
                            let masks = self.masks(input.wall);
                            if let Some(k) = self.king(input) {
                                let bit = self.board.unwrap().cell(k.x, k.y).bit();
                                if masks[piece] & bit != 0 {
                                    self.label(2, input.tick, &mut out)
                                }
                            }
                            if self.pieces[piece].kind == Kind::Knight {
                                let attacks = masks[piece];
                                let b = self.board.unwrap();
                                let targets = input
                                    .fighters
                                    .iter()
                                    .filter(|f| {
                                        f.champion
                                            && f.team != input.owner.team
                                            && f.hp > 0
                                            && f.visible
                                            && attacks & b.cell(f.x, f.y).bit() != 0
                                    })
                                    .count();
                                if targets >= 2 {
                                    self.label(4, input.tick, &mut out)
                                }
                            }
                            self.pin_label(input, &mut out);
                            self.challenge_event(input, 0, piece, None, &mut out);
                            for id in &p.hit {
                                self.challenge_event(input, 1, piece, Some(*id), &mut out);
                            }
                            let b = self.board.unwrap();
                            let discovered = input
                                .fighters
                                .iter()
                                .filter(|f| f.team != input.owner.team && f.visible && f.hp > 0)
                                .any(|f| {
                                    let bit = b.cell(f.x, f.y).bit();
                                    (0..7).any(|i| {
                                        i != piece
                                            && self.pieces[i].kind != Kind::Knight
                                            && masks[i] & bit != 0
                                            && p.before[i] & bit == 0
                                    })
                                });
                            if discovered {
                                self.label(5, input.tick, &mut out);
                            }
                            let new_bishop_check = self.king(input).is_some_and(|k| {
                                let bit = b.cell(k.x, k.y).bit();
                                (2..4).any(|i| {
                                    self.challenge.as_ref().is_some_and(|c| c.a != Some(i))
                                        && masks[i] & bit != 0
                                        && p.before[i] & bit == 0
                                })
                            });
                            if self.pieces[piece].kind == Kind::Knight && new_bishop_check {
                                for id in &p.hit {
                                    self.challenge_event(input, 4, piece, Some(*id), &mut out);
                                }
                            }
                        }
                    } else {
                        self.pending = Some(p);
                    }
                } else {
                    self.pending = Some(p);
                }
            }
        }
        self.checkmate(input, &mut out);
        if self.pending.is_none() && !input.controlled {
            if let Some(action) = input.action {
                if action == Action::Dismiss {
                    self.dismiss(&mut out);
                } else {
                    self.begin(input, action);
                }
            } else if input.automatic && input.tick.is_multiple_of(6) {
                if let Some(a) = self.plan(input) {
                    self.begin(input, a);
                }
            }
        }
        self.coverage = self.masks(input.wall);
        if let Some(b) = self.board {
            let mask = self.warning_cells();
            let start = self
                .pending
                .as_ref()
                .map_or(self.mate_warning.unwrap_or(0), |p| p.start);
            if self.warn_key != (start, mask) {
                self.warn_key = (start, mask);
                self.warn_drawn = 0;
            }
            if input.tick.is_multiple_of(2) {
                let tag = if self.mate_warning.is_some() {
                    "warn24"
                } else if self.pending.as_ref().is_some_and(|p| p.end - p.start >= 36) {
                    "warn45"
                } else {
                    "warn30"
                };
                for n in 0..64 {
                    if mask & !self.warn_drawn & (1u64 << n) != 0 {
                        let (x, y) = b.world(Cell::new((n % 8) as i8, (n / 8) as i8));
                        if self.fx(
                            input.tick,
                            tag,
                            x,
                            y,
                            if tag == "warn45" { 27 } else { 24 },
                            false,
                            &mut out,
                        ) {
                            self.warn_drawn |= 1u64 << n;
                        }
                    }
                }
            }
        }
        out
    }
    pub fn plan(&mut self, input: &Input) -> Option<Action> {
        let mut planning = self.clone();
        planning.cache_geometry(input.wall);
        let action = planning.plan_cached(input);
        self.rng = planning.rng;
        action
    }
    fn plan_cached(&mut self, input: &Input) -> Option<Action> {
        let enemy = input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && f.targetable)
            .min_by_key(|f| {
                let dx = f.x - input.owner.x;
                let dy = f.y - input.owner.y;
                ((dx as i128).pow(2) + (dy as i128).pow(2), f.id)
            });
        let Some(foe) = enemy else { return None };
        let needs_king = self.board.is_some_and(|b| {
            self.king(input).is_none()
                && input.fighters.iter().any(|f| {
                    f.champion
                        && f.team != input.owner.team
                        && f.hp > 0
                        && f.visible
                        && b.inside(f.x, f.y)
                })
        });
        let preserve_attempt = self.rank >= 2
            && self.challenge.as_ref().is_some_and(|c| {
                !c.done && !c.failed && c.started.is_some_and(|t| input.tick <= t + WINDOW)
            });
        let restore = !preserve_attempt
            && input.owner.hp * 100 > input.owner.max_hp * 35
            && [0, 6]
                .iter()
                .any(|&i| self.pieces[i].hp * 2 < self.pieces[i].max_hp);
        if self.board.is_none()
            || self.board.is_some_and(|b| {
                !b.control(input.owner.x, input.owner.y) || !b.inside(foe.x, foe.y)
            })
            || (input.tick >= self.reset_at && (needs_king || restore))
        {
            if input.tick < self.reset_at {
                return None;
            }
            let (dx, dy) = (foe.x - input.owner.x, foe.y - input.owner.y);
            let h = if dy.abs() >= dx.abs() {
                if dy >= 0 {
                    0
                } else {
                    2
                }
            } else if dx >= 0 {
                1
            } else {
                3
            };
            for k in 0..4 {
                let heading = (h + k) % 4;
                let b = Board {
                    x: input.owner.x,
                    y: input.owner.y,
                    heading,
                    serial: 0,
                };
                if b.inside(foe.x, foe.y)
                    && (0..8).all(|x| {
                        let p = b.world(Cell::new(x, 0));
                        !(input.wall)(p.0, p.1)
                    })
                {
                    return Some(Action::Deploy { heading });
                }
            }
            return None;
        }
        let b = self.board.unwrap();
        let me = b.cell(input.owner.x, input.owner.y);
        if input.owner.hp * 100 < input.owner.max_hp * 35 && input.tick >= self.castle_at {
            let choices = (0..2)
                .filter(|&i| self.pieces[i].hp > 0)
                .filter(|&i| {
                    let c = self.pieces[i].cell;
                    let (dx, dy) = ((c.x - me.x).abs(), (c.y - me.y).abs());
                    (dx == 0 || dy == 0) && dx + dy > 0 && dx + dy <= 3
                })
                .collect::<Vec<_>>();
            if let Some(i) = choices
                .into_iter()
                .filter(|&i| self.castle_legal(input, i, me))
                .max_by_key(|&i| {
                    let (x, y) = b.world(self.pieces[i].cell);
                    (x - foe.x).abs() + (y - foe.y).abs()
                })
            {
                return Some(Action::Castle { piece: i });
            }
        }
        let mut choices = vec![];
        for i in 0..7 {
            for n in 0..64 {
                let to = Cell::new(n % 8, n / 8);
                if !self.legal(i, to, me, input.wall) {
                    continue;
                }
                let mut score = self.score(input, i, to);
                // Lower ranks sometimes overlook the best continuation, using isolated seeded choice noise.
                if self.rank < 6 {
                    score += self.roll((6 - self.rank) * 90 + 1) as i64
                }
                choices.push((score, i, to));
            }
        }
        choices.sort_by_key(|(s, i, c)| (-*s, *i, c.index()));
        let depth = [1, 1, 2, 2, 3, 3, 4, 4][self.rank];
        if depth == 1 {
            return choices
                .first()
                .filter(|c| c.0 > 0)
                .map(|(_, piece, to)| Action::Move {
                    piece: *piece,
                    to: *to,
                });
        }
        let mut beam: Vec<_> = choices
            .iter()
            .take(3)
            .map(|&(s, i, to)| {
                let mut c = self.clone();
                c.pieces[i].cell = to;
                c.pending = None;
                (s, s, Action::Move { piece: i, to }, c)
            })
            .collect();
        // Three positions, eight continuations per position, up to four actual moves.
        for ply in 1..depth {
            let dt = (ply * 30) as i64;
            let mut predicted = input.fighters.to_vec();
            for f in predicted
                .iter_mut()
                .filter(|f| f.visible && f.team != input.owner.team)
            {
                if let Some(v) = self.seen.iter().find(|s| s.0 == f.id && s.3 == input.tick) {
                    let x = f.x + v.4 * dt * self.rank as i64 / 7;
                    let y = f.y + v.5 * dt * self.rank as i64 / 7;
                    if !(input.wall)(x, y) {
                        f.x = x;
                        f.y = y
                    }
                }
            }
            let future_input = Input {
                tick: input.tick + dt as usize,
                owner: input.owner,
                fighters: &predicted,
                wall: input.wall,
                captures: &[],
                piece_hp: [None; 7],
                controlled: false,
                automatic: false,
                action: None,
            };
            let mut next = vec![];
            for (total, first_score, first, c) in beam {
                let mut continuations = vec![];
                for i in 0..7 {
                    for n in 0..64 {
                        let to = Cell::new(n % 8, n / 8);
                        if c.legal(i, to, me, input.wall) {
                            continuations.push((c.score(&future_input, i, to), i, to))
                        }
                    }
                }
                continuations.sort_by_key(|(s, i, to)| (-*s, *i, to.index()));
                for (s, i, to) in continuations.into_iter().take(8) {
                    let mut after = c.clone();
                    after.pieces[i].cell = to;
                    next.push((total + s / (ply as i64 + 1), first_score, first, after));
                }
            }
            next.sort_by_key(|v| -v.0);
            next.truncate(3);
            beam = next;
        }
        beam.first().filter(|v| v.1 > 0).map(|v| v.2)
    }
    fn score(&self, input: &Input, piece: usize, to: Cell) -> i64 {
        let b = self.board.unwrap();
        let old = self.pieces[piece].cell;
        let kind = self.pieces[piece].kind;
        let mut score = 0;
        let mut hit_ids = vec![];
        for f in input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && b.inside(f.x, f.y))
        {
            let c = b.cell(f.x, f.y);
            let on_path = if kind == Kind::Knight {
                c == to
            } else {
                let dx = (to.x - old.x).signum();
                let dy = (to.y - old.y).signum();
                let len = (to.x - old.x).abs().max((to.y - old.y).abs());
                (1..=len).any(|j| Cell::new(old.x + dx * j, old.y + dy * j) == c)
            };
            if on_path {
                hit_ids.push(f.id);
                score += if f.champion { 450 } else { 80 };
                score += kind.damage(input.owner.ap).min(f.hp) / 2
            }
        }
        let mut future = self.clone();
        future.pieces[piece].cell = to;
        future.pending = None;
        let masks = future.masks(input.wall);
        let all = masks.iter().fold(0, |a, b| a | b);
        if let Some(k) = self.king(input) {
            let c = b.cell(k.x, k.y);
            let trapped = all & c.bit() != 0
                && (-1..=1).all(|dx| {
                    (-1..=1).all(|dy| {
                        if dx == 0 && dy == 0 {
                            return true;
                        }
                        let escape = Cell::new(c.x + dx, c.y + dy);
                        let (x, y) = b.world(escape);
                        (input.wall)(x, y) || (escape.valid() && all & escape.bit() != 0)
                    })
                });
            if trapped
                && masks
                    .iter()
                    .filter(|m| **m & (c.bit() | neighbors(c)) != 0)
                    .count()
                    >= 2
            {
                score += 900 + self.rank as i64 * 100;
            }
        }
        if let Some(k) = self.king(input) {
            let c = b.cell(k.x, k.y);
            score += ((all & neighbors(c)).count_ones() as i64) * (12 + self.rank as i64 * 8);
            if all & c.bit() != 0 {
                score += 80 + self.rank as i64 * 25;
            }
        }
        let protect = self
            .pieces
            .iter()
            .enumerate()
            .filter(|(i, p)| *i != piece && p.hp > 0 && masks[*i] & to.bit() != 0)
            .count();
        score += protect as i64 * (8 + self.rank as i64 * 6);
        if let Some(c) = &self.challenge {
            if !c.done && !c.failed {
                let want = match (c.id, c.progress) {
                    (0, 0) => piece == 6,
                    (0, 2) => Some(piece) == c.a,
                    (1, 0 | 2) => piece < 2,
                    (1, 1) => Some(piece) == c.b,
                    (2, 0) => piece < 4,
                    (2, 2) => piece < 2,
                    (3, 0) => kind == Kind::Bishop,
                    (3, 2) => kind == Kind::Knight,
                    (4, 0) => kind == Kind::Knight,
                    (4, 2) => kind == Kind::Bishop,
                    (5, 0 | 2) => kind == Kind::Rook,
                    (5, 3) => kind == Kind::Queen,
                    _ => false,
                };
                if want {
                    score += self.rank as i64 * 18;
                }
                // Recognise a geometrically valid next event, including its identities,
                // rather than awarding planning value merely for the right piece type.
                let old_progress = c.progress;
                let mut commands = vec![];
                future.challenge_event(input, 0, piece, None, &mut commands);
                let before = self.masks(input.wall);
                for id in hit_ids {
                    future.challenge_event(input, 1, piece, Some(id), &mut commands);
                    let discovery = self.king(input).is_some_and(|k| {
                        let bit = b.cell(k.x, k.y).bit();
                        (2..4)
                            .any(|i| Some(i) != c.a && masks[i] & bit != 0 && before[i] & bit == 0)
                    });
                    if kind == Kind::Knight && discovery {
                        future.challenge_event(input, 4, piece, Some(id), &mut commands);
                    }
                }
                if let Some(after) = &future.challenge {
                    score += (after.progress.saturating_sub(old_progress) as i64)
                        * self.rank as i64
                        * 70;
                }
            }
        }
        let (x, y) = b.world(to);
        let near = input
            .fighters
            .iter()
            .filter(|f| {
                f.team != input.owner.team
                    && f.hp > 0
                    && f.visible
                    && (f.x - x).abs() + (f.y - y).abs() < TILE * 2
            })
            .count();
        let deliberate = future.pieces[piece].exposed
            && future
                .challenge
                .as_ref()
                .is_some_and(|c| !c.done && !c.failed && c.progress > 0);
        let exposure_cost =
            near as i64 * (self.rank as i64 * 10) + (if kind == Kind::Queen { 65 } else { 15 });
        score -= if deliberate {
            exposure_cost / 2
        } else {
            exposure_cost
        };
        score
    }
    pub fn warning_cells(&self) -> u64 {
        let mate = self
            .mate_cell
            .filter(|_| self.mate_warning.is_some())
            .map_or(0, |c| c.bit() | neighbors(c));
        let Some(p) = &self.pending else { return mate };
        mate | match p.action {
            Action::Move { piece, to } => {
                if self.pieces[piece].kind == Kind::Knight {
                    to.bit()
                } else {
                    let (dx, dy) = ((to.x - p.from.x).signum(), (to.y - p.from.y).signum());
                    let n = (to.x - p.from.x).abs().max((to.y - p.from.y).abs());
                    (1..=n).fold(0, |mask, j| {
                        mask | Cell::new(p.from.x + dx * j, p.from.y + dy * j).bit()
                    })
                }
            }
            _ => 0,
        }
    }
    fn pin_label(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        let Some(b) = self.board else { return };
        let Some(k) = self.king(input) else { return };
        let kc = b.cell(k.x, k.y);
        for i in [0, 1, 2, 3, 6] {
            let p = &self.pieces[i];
            if p.hp <= 0 {
                continue;
            }
            let (dx, dy) = (kc.x - p.cell.x, kc.y - p.cell.y);
            if dx == 0 && dy == 0 {
                continue;
            }
            let straight = dx == 0 || dy == 0;
            let diagonal = dx.abs() == dy.abs();
            if !(match p.kind {
                Kind::Rook => straight,
                Kind::Bishop => diagonal,
                Kind::Queen => straight || diagonal,
                _ => false,
            }) {
                continue;
            }
            let mut c = p.cell;
            let mut enemies = BTreeSet::new();
            let mut blocked = false;
            while c != kc {
                c = Cell::new(c.x + dx.signum(), c.y + dy.signum());
                let (x, y) = b.world(c);
                if (input.wall)(x, y)
                    || c == self.owner_cell
                    || self.external_blockers & c.bit() != 0
                    || self
                        .pieces
                        .iter()
                        .enumerate()
                        .any(|(j, q)| j != i && q.hp > 0 && q.cell == c)
                {
                    blocked = true;
                    break;
                }
                if c != kc {
                    for f in input.fighters.iter().filter(|f| {
                        f.champion
                            && f.hp > 0
                            && f.team != input.owner.team
                            && f.visible
                            && b.cell(f.x, f.y) == c
                    }) {
                        enemies.insert(f.id);
                    }
                }
            }
            if !blocked && enemies.len() == 1 {
                self.label(11, input.tick, out);
                return;
            }
        }
    }
    #[allow(dead_code)] // Used by the shared lab and native parity runner.
    pub fn json(&self, commands: &[Cmd]) -> String {
        let mut out = String::new();
        let b = self.board;
        let _ = write!(
            out,
            "{{\"owner\":{},\"rank\":{},\"board\":",
            self.owner, self.rank
        );
        if let Some(b) = b {
            let _ = write!(
                out,
                "{{\"x\":{},\"y\":{},\"heading\":{},\"serial\":{}}}",
                b.x, b.y, b.heading, b.serial
            );
        } else {
            out.push_str("null")
        }
        let _=write!(out,",\"rankName\":\"{}\"",RANKS[self.rank]);
        out.push_str(",\"pieces\":[");
        for (i, p) in self.pieces.iter().enumerate() {
            if i > 0 {
                out.push(',')
            }
            let _=write!(out,"{{\"id\":{},\"kind\":{},\"x\":{},\"y\":{},\"homeX\":{},\"hp\":{},\"maxHp\":{},\"readyAt\":{},\"exposed\":{}}}",i,p.kind.code(),p.cell.x,p.cell.y,p.home.x,p.hp,p.max_hp,p.ready_at,p.exposed);
        }
        out.push_str("],\"projections\":[");
        for (i, p) in self.projections.iter().enumerate() {
            if i > 0 {
                out.push(',')
            }
            let _ = write!(out, "{{\"id\":{},\"role\":{}}}", p.id, p.role.code());
        }
        out.push(']');
        let _ = write!(
            out,
            ",\"glowMask\":{},\"glowUntil\":{}",
            self.glow_mask, self.glow_until
        );
        let _ = write!(
            out,
            ",\"state\":\"{:?}\",\"coverage\":[{}]",
            self,
            self.coverage
                .iter()
                .map(|m| format!("\"{m:016x}\""))
                .collect::<Vec<_>>()
                .join(",")
        );
        let _=write!(out,",\"resetAt\":{},\"castleAt\":{},\"mateAt\":{},\"rewardAt\":{},\"brilliantUntil\":{},\"mateUsed\":{},\"hasMove\":{},\"mateWarning\":{},\"label\":{},\"labelUntil\":{},\"blockedLo\":{},\"blockedHi\":{},\"warningLo\":{},\"warningHi\":{}",
            self.reset_at,self.castle_at,self.mate_at,self.reward_at,self.brilliant_until,self.mate_used,self.has_move,self.mate_warning.map_or("null".into(),|t|t.to_string()),self.last_label,self.label_until,self.blocked as u32,self.blocked>>32,self.warning_cells() as u32,self.warning_cells()>>32);
        out.push_str(",\"challenge\":");
        if let Some(c) = &self.challenge {
            let _=write!(out,"{{\"id\":{},\"progress\":{},\"started\":{},\"done\":{},\"failed\":{},\"failure\":{}}}",c.id,c.progress,c.started.map_or("null".into(),|t|t.to_string()),c.done,c.failed,c.failure);
        } else {
            out.push_str("null")
        }
        out.push_str(",\"pending\":");
        if let Some(p) = &self.pending {
            let (a, i, x, y) = match p.action {
                Action::Deploy { heading } => (0, heading as i64, 0, 0),
                Action::Move { piece, to } => (1, piece as i64, to.x as i64, to.y as i64),
                Action::Castle { piece } => (2, piece as i64, 0, 0),
                Action::Dismiss => (3, 0, 0, 0),
            };
            let _=write!(out,"{{\"type\":{},\"piece\":{},\"x\":{},\"y\":{},\"start\":{},\"commit\":{},\"end\":{},\"committed\":{}}}",a,i,x,y,p.start,p.commit,p.end,p.committed);
        } else {
            out.push_str("null")
        }
        let _=write!(out,",\"stats\":{{\"commands\":{:?},\"captures\":{},\"mates\":{},\"rewards\":{},\"resets\":{},\"damage\":{},\"castles\":{},\"cancelled\":{}}},\"commands\":[",
            self.stats.commands,self.stats.captures,self.stats.mates,self.stats.rewards,self.stats.resets,self.stats.damage,self.stats.casts,self.stats.cancelled);
        for (i, c) in commands.iter().enumerate() {
            if i > 0 {
                out.push(',')
            }
            match c {
                Cmd::Spawn { piece, x, y, hp } => {
                    let _ = write!(
                        out,
                        "{{\"type\":\"spawn\",\"piece\":{piece},\"x\":{x},\"y\":{y},\"hp\":{hp}}}"
                    );
                }
                Cmd::Despawn { piece } => {
                    let _ = write!(out, "{{\"type\":\"despawn\",\"piece\":{piece}}}");
                }
                Cmd::Move { piece, x, y } => {
                    let _ = write!(
                        out,
                        "{{\"type\":\"move\",\"piece\":{piece},\"x\":{x},\"y\":{y}}}"
                    );
                }
                Cmd::Damage {
                    piece,
                    target,
                    amount,
                } => {
                    let _=write!(out,"{{\"type\":\"damage\",\"piece\":{piece},\"target\":{target},\"amount\":{amount}}}");
                }
                Cmd::Stun { target, ticks } => {
                    let _ = write!(
                        out,
                        "{{\"type\":\"stun\",\"target\":{target},\"ticks\":{ticks}}}"
                    );
                }
                Cmd::Shield { amount, ticks } => {
                    let _ = write!(
                        out,
                        "{{\"type\":\"shield\",\"amount\":{amount},\"ticks\":{ticks}}}"
                    );
                }
                Cmd::OwnerPos { x, y } => {
                    let _ = write!(out, "{{\"type\":\"owner\",\"x\":{x},\"y\":{y}}}");
                }
                Cmd::Fx { tag, x, y, ticks } => {
                    let _=write!(out,"{{\"type\":\"fx\",\"tag\":\"{tag}\",\"x\":{x},\"y\":{y},\"ticks\":{ticks}}}");
                }
                Cmd::Label { code } => {
                    let _ = write!(out, "{{\"type\":\"label\",\"code\":{code}}}");
                }
            }
        }
        out.push_str("]}");
        out
    }
}
pub fn neighbors(c: Cell) -> u64 {
    let mut mask = 0;
    for x in -1..=1 {
        for y in -1..=1 {
            if x != 0 || y != 0 {
                mask |= Cell::new(c.x + x, c.y + y).bit()
            }
        }
    }
    mask
}

// An explicit mention keeps the generated catalogue part of both builds and catches missing entries.
#[cfg(test)]
pub fn catalogue_count() -> usize {
    GAMES.len()
}

#[cfg(test)]
#[path = "magnus_tests.rs"]
mod tests;
