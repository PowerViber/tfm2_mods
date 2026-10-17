//! Full legal command trajectories; replays never inject progress or rewards.
use crate::magnus_core::*;
use crate::magnus_protocol::Frame;
pub fn owner() -> Fighter {
    Fighter {
        id: 1,
        team: 0,
        x: 16 * TILE,
        y: 12 * TILE,
        hp: 800,
        max_hp: 800,
        attack: 45,
        ap: 50,
        champion: true,
        farmable: true,
        visible: true,
        targetable: true,
        tank: false,
        ranged: true,
        friendly_chess: false,
        sight_source: true,
        mate_immune_until: 0,
        vx: 0, vy: 0, move_speed: 0,
    }
}
pub fn enemy(b: Board, id: usize, c: Cell) -> Fighter {
    let (x, y) = b.world(c);
    Fighter {
        id,
        team: 1,
        x,
        y,
        hp: 20_000,
        max_hp: 20_000,
        attack: 45,
        ap: 50,
        champion: true,
        farmable: true,
        visible: true,
        targetable: true,
        tank: false,
        ranged: false,
        friendly_chess: false,
        sight_source: false,
        mate_immune_until: 0,
        vx: 0, vy: 0, move_speed: 0,
    }
}
pub struct Fixture {
    pub core: Core,
    pub frame: Frame,
    pub moves: Vec<(usize, Action)>,
    pub captures: Vec<(usize, usize)>,
    pub relocate: Vec<(usize, usize, Cell)>,
    pub walk: Vec<(usize, i64, i64)>,
    pub end: usize,
}
impl Fixture {
    pub fn prepare(&mut self, tick: usize) {
        self.frame.tick = tick;
        self.frame.action = self.moves.iter().find(|(t, _)| *t == tick).map(|(_, a)| *a);
        for (_, id, c) in self.relocate.iter().filter(|(t, _, _)| *t == tick) {
            if let Some(f) = self.frame.fighters.iter_mut().find(|f| f.id == *id) {
                (f.x, f.y) = c.world();
            }
        }
        for (_, x, y) in self.walk.iter().filter(|(t, _, _)| *t == tick) {
            self.frame.owner.x = *x;
            self.frame.owner.y = *y;
        }
        self.frame
            .captures
            .extend(
                self.captures
                    .iter()
                    .filter(|(t, _)| *t == tick)
                    .map(|(_, id)| Capture {
                        piece: *id,
                        by: Some(5),
                        combat: true,
                    }),
            );
    }
    pub fn advance(&mut self, tick: usize) -> (String, Vec<Cmd>) {
        self.prepare(tick);
        let result = self.frame.run(&mut self.core);
        self.frame.apply(&result.1);
        result
    }
    pub fn json(&self) -> String {
        let mut out = format!(
            "{{\"state\":{},\"values\":{:?},\"moves\":[",
            self.frame.state(&self.core),
            self.frame.values()
        );
        for (n, (t, a)) in self.moves.iter().enumerate() {
            if n > 0 {
                out.push(',');
            }
            let (k, p, x, y, h) = action_key(*a);
            out.push_str(&format!("[{t},{k},{p},{x},{y},{h}]"));
        }
        out.push_str("],\"captures\":[");
        for (n, (t, p)) in self.captures.iter().enumerate() {
            if n > 0 {
                out.push(',');
            }
            out.push_str(&format!("[{t},{p}]"));
        }
        out.push_str("],\"relocate\":[");
        for (n, (t, id, c)) in self.relocate.iter().enumerate() {
            if n > 0 {
                out.push(',');
            }
            let (x, y) = c.world();
            out.push_str(&format!("[{t},{id},{x},{y}]"));
        }
        out.push_str("],\"walk\":[");
        for (n, (tick, x, y)) in self.walk.iter().enumerate() {
            if n > 0 {
                out.push(',');
            }
            out.push_str(&format!("[{tick},{x},{y}]"));
        }
        out.push_str(&format!("],\"end\":{}}}", self.end));
        out
    }
}
fn install(c: &mut Core, b: Board, id: usize, kind: Kind, x: i32, y: i32) {
    let slot = (id < 7).then_some(id);
    let p = Piece::new(
        id,
        kind,
        b.grid(Cell::new(x, y)),
        800 * kind.hp_pct() / 100,
        slot,
        0,
    );
    c.pieces.insert(id, p);
    c.next_id = c.next_id.max(id + 1);
    if let Some(s) = slot {
        c.slots[s] = Some(id);
    }
    c.reindex();
}
pub fn fixture(id: usize) -> Option<Fixture> {
    if id >= 12 {
        return None;
    }
    let me = owner();
    let b = Board {
        x: me.x,
        y: me.y,
        heading: 0,
        serial: 1,
    };
    let mut core = Core::new(123, 1, 3);
    core.board = Some(b);
    core.stats.resets = 1;
    core.reset_at = 1440;
    core.previous_challenge = Some(id);
    let mut frame = Frame::new(0, me);
    frame.fighters = vec![enemy(b, 2, Cell::new(6, 1)), enemy(b, 3, Cell::new(6, 6))];
    core.projections = vec![
        Projection {
            id: 2,
            role: Role::King,
        },
        Projection {
            id: 3,
            role: Role::Queen,
        },
    ];
    let (mut moves, mut caps, mut relocate, mut walks) = (vec![], vec![], vec![], vec![]);
    let mut add = |i, x, y| install(&mut core, b, i, PIECE_KINDS[i], x, y);
    match id {
        0 => {
            add(0, 0, 6);
            add(6, 3, 3);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 5));
            moves = vec![(0, 6, 0, 3, 6), (47, 0, 0, 0, 5), (78, 0, 0, 4, 5)];
            caps.push((46, 6));
        }
        1 => {
            add(0, 4, 6);
            add(4, 2, 3);
            frame.fighters[1] = enemy(b, 3, Cell::new(4, 4));
            moves = vec![
                (0, 0, 0, 4, 3),
                (31, 4, 0, 4, 4),
                (63, 0, 0, 6, 3),
                (94, 0, 0, 6, 7),
                (125, 0, 0, 4, 7),
                (156, 0, 0, 4, 5),
            ];
            relocate.push((63, 3, b.grid(Cell::new(4, 6))));
        }
        2 => {
            add(0, 3, 4);
            add(2, 5, 6);
            add(6, 7, 7);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 3));
            moves.push((0, 2, 0, 4, 5));
            caps.push((31, 6));
            frame.walls = trap(b, Cell::new(3, 3), &[Cell::new(3, 4)]);
        }
        3 => {
            add(2, 0, 1);
            add(3, 5, 5);
            add(4, 4, 4);
            add(6, 7, 6);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 4));
            frame.fighters.push(enemy(b, 4, Cell::new(6, 5)));
            frame.fighters.push(enemy(b, 5, Cell::new(7, 6)));
            moves = vec![(0, 2, 0, 1, 2), (32, 4, 0, 6, 5)];
            caps.push((31, 6));
            relocate.push((31, 2, b.grid(Cell::new(3, 3))));
        }
        4 => {
            add(2, 5, 5);
            add(4, 0, 1);
            add(5, 5, 3);
            add(6, 7, 7);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 3));
            moves.push((0, 4, 0, 1, 3));
            caps.push((31, 6));
            frame.walls = trap(
                b,
                Cell::new(3, 3),
                &[Cell::new(3, 2), Cell::new(3, 4), Cell::new(4, 4)],
            );
        }
        5 => {
            add(0, 0, 2);
            add(1, 0, 5);
            add(6, 7, 6);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 5));
            frame.fighters.push(enemy(b, 4, Cell::new(2, 2)));
            moves = vec![(0, 0, 0, 4, 2), (32, 1, 0, 4, 5), (64, 6, 0, 4, 6)];
            caps.push((31, 0));
            relocate.push((63, 2, b.grid(Cell::new(5, 6))));
        }
        6 => {
            add(2, 0, 1);
            add(3, 6, 1);
            add(6, 7, 4);
            add(0, 7, 6);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 4));
            moves = vec![
                (0, 2, 0, 4, 5),
                (33, 6, 0, 2, 4),
                (80, 3, 0, 2, 5),
                (113, 0, 0, 3, 6),
                (145, 0, 0, 3, 3),
            ];
            caps = vec![(32, 2), (112, 3)];
        }
        7 => {
            add(0, 0, 2);
            add(1, 0, 5);
            add(6, 7, 7);
            add(2, 6, 2);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 5));
            frame.fighters.push(enemy(b, 4, Cell::new(2, 2)));
            moves = vec![(0, 0, 0, 4, 2), (33, 1, 0, 4, 5), (65, 2, 0, 2, 6)];
            caps.push((32, 6));
        }
        8 => {
            add(2, 0, 0);
            add(6, 1, 7);
            install(&mut core, b, 7, Kind::Pawn, 1, 4);
            frame.fighters[0] = enemy(b, 2, Cell::new(5, 5));
            frame.fighters[1] = enemy(b, 3, Cell::new(3, 3));
            moves = vec![
                (0, 2, 0, 4, 4),
                (33, 7, 0, 1, 5),
                (65, 7, 0, 1, 6),
                (97, 6, 0, 5, 7),
                (144, 6, 0, 5, 4),
            ];
            caps.push((32, 2));
        }
        9 => {
            add(6, 3, 7);
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 4));
            let (x, y) = b.world(Cell::new(4, 1));
            frame.owner.x = x;
            frame.owner.y = y;
            walks = walk(
                b,
                &[
                    (1, Cell::new(4, 1), Cell::new(4, 2)),
                    (26, Cell::new(4, 2), Cell::new(4, 3)),
                ],
            );
            moves.push((52, 6, 0, 3, 3));
        }
        10 => {
            install(&mut core, b, 7, Kind::Pawn, 1, 4);
            core.pieces.get_mut(&7).unwrap().ranks = 4;
            core.pieces.get_mut(&7).unwrap().promotion = Kind::Rook;
            frame.fighters[0] = enemy(b, 2, Cell::new(3, 6));
            frame.fighters.push(enemy(b, 4, Cell::new(2, 5)));
            let (x, y) = b.world(Cell::new(1, 5));
            frame.owner.x = x;
            frame.owner.y = y;
            walks = walk(
                b,
                &[
                    (63, Cell::new(1, 5), Cell::new(2, 5)),
                    (88, Cell::new(2, 5), Cell::new(3, 5)),
                    (113, Cell::new(3, 5), Cell::new(4, 5)),
                ],
            );
            moves = vec![(0, 7, 0, 2, 5), (32, 7, 0, 2, 6), (140, 7, 0, 4, 6)];
        }
        11 => {
            install(&mut core, b, 7, Kind::Pawn, 4, 4);
            frame.fighters[0] = enemy(b, 2, Cell::new(5, 6));
            let (x, y) = b.world(Cell::new(1, 1));
            frame.owner.x = x;
            frame.owner.y = y;
            walks = walk(
                b,
                &[
                    (1, Cell::new(1, 1), Cell::new(2, 2)),
                    (26, Cell::new(2, 2), Cell::new(3, 3)),
                ],
            );
            moves = vec![(52, 7, 0, 4, 5), (84, 7, 0, 5, 6)];
        }
        _ => {}
    }
    if !caps.is_empty() && !frame.fighters.iter().any(|f| f.id == 5) {
        let cell = core.pieces.get(&caps[0].1).map_or(Cell::new(7, 7), |p| {
            b.cell(p.cell.world().0, p.cell.world().1)
        });
        frame.fighters.push(enemy(b, 5, cell));
    }
    for f in &frame.fighters {
        if !core.projections.iter().any(|p| p.id == f.id) {
            core.projections.push(Projection {
                id: f.id,
                role: Role::Knight,
            });
        }
    }
    let mut challenge = Challenge::new(id);
    challenge.target = Some(2);
    challenge.queen_target = Some(3);
    challenge.board_serial = Some(1);
    challenge.owner_last = frame.owner.cell();
    core.challenge = Some(challenge);
    core.reindex();
    Some(Fixture {
        core,
        frame,
        moves: moves
            .into_iter()
            .map(|(tick, piece, _, x, y)| {
                (
                    tick,
                    Action::Move {
                        piece,
                        to: b.grid(Cell::new(x, y)),
                    },
                )
            })
            .collect(),
        captures: caps,
        relocate,
        walk: walks,
        end: 220,
    })
}
fn walk(b: Board, legs: &[(usize, Cell, Cell)]) -> Vec<(usize, i64, i64)> {
    let mut v = vec![];
    for (t, a, z) in legs {
        let (ax, ay) = b.world(*a);
        let (zx, zy) = b.world(*z);
        for n in 0..25 {
            v.push((
                t + n,
                ax + (zx - ax) * (n + 1) as i64 / 25,
                ay + (zy - ay) * (n + 1) as i64 / 25,
            ));
        }
    }
    v
}
pub fn trap(b: Board, king: Cell, open: &[Cell]) -> Vec<[i64; 4]> {
    let mut walls = vec![];
    for dx in -1..=1 {
        for dy in -1..=1 {
            let c = Cell::new(king.x + dx, king.y + dy);
            if (dx != 0 || dy != 0) && !open.contains(&c) {
                let (x, y) = b.world(c);
                walls.push([x - 11900, y - 11900, x + 11900, y + 11900]);
            }
        }
    }
    walls
}
