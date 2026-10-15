//! Practice scenarios replay actual movement/capture frames; they do not inject rewards.
use crate::magnus_core::*;
pub fn owner() -> Fighter {
    Fighter {
        id: 1,
        team: 0,
        x: 400_000,
        y: 400_000,
        hp: 800,
        max_hp: 800,
        attack: 45,
        ap: 50,
        champion: true,
        visible: true,
        targetable: true,
        tank: false,
        ranged: true,
        friendly_chess: false,
    }
}
pub fn enemy(b: Board, id: usize, cell: Cell) -> Fighter {
    let (x, y) = b.world(cell);
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
        visible: true,
        targetable: true,
        tank: false,
        ranged: false,
        friendly_chess: false,
    }
}
pub struct Fixture {
    pub core: Core,
    pub me: Fighter,
    pub fighters: Vec<Fighter>,
    pub walls: Vec<[i64; 4]>,
    pub moves: Vec<(usize, usize, Cell)>,
    pub captures: Vec<(usize, usize)>,
    pub relocate: Vec<(usize, usize, Cell)>,
    pub end: usize,
}
pub fn historical(id: usize) -> Fixture {
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
    core.previous_challenge = Some(id);
    core.challenge = Some(Challenge::new(id));
    core.reset_at = 960;
    for p in &mut core.pieces {
        p.hp = 0;
        p.max_hp = 200;
        p.ready_at = 1_000_000_000;
    }
    let mut fighters = vec![enemy(b, 2, Cell::new(6, 1)), enemy(b, 3, Cell::new(6, 6))];
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
    let mut setup = |i: usize, x, y| {
        core.pieces[i].cell = Cell::new(x, y);
        core.pieces[i].hp = 200;
    };
    let (mut moves, mut captures, mut relocate, mut walls) = (vec![], vec![], vec![], vec![]);
    match id {
        0 => {
            setup(0, 0, 6);
            setup(6, 3, 3);
            fighters[0] = enemy(b, 2, Cell::new(3, 5));
            moves = vec![
                (0, 6, Cell::new(3, 6)),
                (47, 0, Cell::new(0, 5)),
                (78, 0, Cell::new(4, 5)),
            ];
            captures.push((46, 6));
        }
        1 => {
            setup(0, 4, 6);
            setup(4, 2, 3);
            fighters[1] = enemy(b, 3, Cell::new(4, 4));
            moves = vec![
                (0, 0, Cell::new(4, 3)),
                (31, 4, Cell::new(4, 4)),
                (63, 0, Cell::new(6, 3)),
                (94, 0, Cell::new(6, 7)),
                (125, 0, Cell::new(4, 7)),
                (156, 0, Cell::new(4, 5)),
            ];
            relocate.push((62, 3, Cell::new(4, 6)));
        }
        2 => {
            setup(0, 3, 4);
            setup(2, 5, 6);
            setup(6, 7, 7);
            fighters[0] = enemy(b, 2, Cell::new(3, 3));
            moves.push((0, 2, Cell::new(4, 5)));
            captures.push((31, 6));
            walls = trap(b, Cell::new(3, 3), &[Cell::new(3, 4)]);
        }
        3 => {
            setup(2, 0, 1);
            setup(3, 5, 5);
            setup(4, 4, 4);
            setup(6, 7, 6);
            fighters[0] = enemy(b, 2, Cell::new(3, 4));
            fighters.push(enemy(b, 4, Cell::new(6, 5)));
            fighters.push(enemy(b, 5, Cell::new(7, 6)));
            moves = vec![(0, 2, Cell::new(1, 2)), (32, 4, Cell::new(6, 5))];
            captures.push((31, 6));
            relocate.push((31, 2, Cell::new(3, 3)));
        }
        4 => {
            setup(2, 5, 5);
            setup(4, 0, 1);
            setup(5, 5, 3);
            setup(6, 7, 7);
            fighters[0] = enemy(b, 2, Cell::new(3, 3));
            moves.push((0, 4, Cell::new(1, 3)));
            captures.push((31, 6));
            walls = trap(
                b,
                Cell::new(3, 3),
                &[Cell::new(3, 2), Cell::new(3, 4), Cell::new(4, 4)],
            );
        }
        5 => {
            setup(0, 0, 2);
            setup(1, 0, 5);
            setup(6, 7, 6);
            fighters[0] = enemy(b, 2, Cell::new(3, 5));
            fighters.push(enemy(b, 4, Cell::new(2, 2)));
            moves = vec![
                (0, 0, Cell::new(4, 2)),
                (32, 1, Cell::new(4, 5)),
                (64, 6, Cell::new(4, 6)),
            ];
            captures.push((31, 0));
            relocate.push((63, 2, Cell::new(5, 6)));
        }
        _ => {}
    }
    if !captures.is_empty() && !fighters.iter().any(|f| f.id == 5) {
        let c = if id == 0 {
            Cell::new(3, 6)
        } else if id == 5 {
            Cell::new(4, 2)
        } else {
            core.pieces[6].cell
        };
        fighters.push(enemy(b, 5, c));
    }
    Fixture {
        core,
        me,
        fighters,
        walls,
        moves,
        captures,
        relocate,
        end: 220,
    }
}
pub fn trap(b: Board, king: Cell, open: &[Cell]) -> Vec<[i64; 4]> {
    let mut walls = vec![];
    for dx in -1..=1 {
        for dy in -1..=1 {
            let c = Cell::new(king.x + dx, king.y + dy);
            if (dx != 0 || dy != 0) && !open.contains(&c) {
                let (x, y) = b.world(c);
                walls.push([x - 11_900, y - 11_900, x + 11_900, y + 11_900]);
            }
        }
    }
    walls
}
