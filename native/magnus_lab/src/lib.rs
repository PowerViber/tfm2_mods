//! Small checked transport around the exact native Magnus core; no second combat implementation.
#![allow(dead_code)]
#[path = "../../tfm2_custom_ai/src/magnus_core.rs"]
mod magnus_core;
#[path = "../../tfm2_custom_ai/src/magnus_data.rs"]
mod magnus_data;
#[path = "../../tfm2_custom_ai/src/magnus_fixtures.rs"]
mod magnus_fixtures;
use magnus_core::*;
use std::cell::RefCell;

#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    input: Vec<i64>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
#[no_mangle]
pub extern "C" fn magnus_version() -> u32 {
    1
}
#[no_mangle]
pub extern "C" fn magnus_new(lo: u32, hi: u32, owner: u32, rank: u32) -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let c = Core::new((hi as u64) << 32 | lo as u64, owner as usize, rank as usize);
        if let Some(i) = s.cores.iter().position(Option::is_none) {
            s.cores[i] = Some(c);
            i as u32
        } else {
            s.cores.push(Some(c));
            (s.cores.len() - 1) as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn magnus_drop(handle: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(handle as usize) {
            *c = None
        }
    })
}
#[no_mangle]
pub extern "C" fn magnus_input(len: u32) -> *mut i64 {
    if len > 8192 {
        return std::ptr::null_mut();
    }
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        s.input.resize(len as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn magnus_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
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
    }
}
#[no_mangle]
pub extern "C" fn magnus_step(handle: u32, len: u32) -> *const u8 {
    SESSION.with(|session| {
        let mut s = session.borrow_mut();
        let result = (|| -> Option<String> {
            let a = s.input.get(..len as usize)?.to_vec();
            if a.len() < 27 {
                return None;
            }
            let nf = usize::try_from(a[17]).ok()?;
            let nr = usize::try_from(a[18]).ok()?;
            let nc = usize::try_from(a[19]).ok()?;
            if nf > 256 || nr > 256 || nc > 7 || a.len() != 27 + nf * 12 + nr * 4 + nc * 3 {
                return None;
            }
            let core = s.cores.get_mut(handle as usize)?.as_mut()?;
            let owner = Fighter {
                id: core.owner,
                team: a[2].max(0) as usize,
                x: a[3],
                y: a[4],
                hp: if a[1] != 0 { a[5] } else { 0 },
                max_hp: a[6].max(1),
                attack: a[7],
                ap: a[8],
                champion: true,
                visible: true,
                targetable: true,
                tank: false,
                ranged: true,
                friendly_chess: false,
            };
            core.rank = (a[16].max(0) as usize).min(7);
            let fighters = a[27..27 + nf * 12]
                .chunks_exact(12)
                .map(fighter)
                .collect::<Vec<_>>();
            let rect_start = 27 + nf * 12;
            let rectangles = &a[rect_start..rect_start + nr * 4];
            let wall = |x: i64, y: i64| {
                x < 0
                    || y < 0
                    || x >= 960_000
                    || y >= 960_000
                    || rectangles
                        .chunks_exact(4)
                        .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
            };
            let captures = a[rect_start + nr * 4..]
                .chunks_exact(3)
                .map(|v| Capture {
                    piece: v[0].max(0) as usize,
                    by: if v[1] < 0 { None } else { Some(v[1] as usize) },
                    combat: v[2] != 0,
                })
                .collect::<Vec<_>>();
            let action = match a[11] {
                0 => Some(Action::Deploy {
                    heading: a[15].rem_euclid(4) as u8,
                }),
                1 => Some(Action::Move {
                    piece: a[12].max(0) as usize,
                    to: Cell::new(a[13].clamp(-120, 120) as i8, a[14].clamp(-120, 120) as i8),
                }),
                2 => Some(Action::Castle {
                    piece: a[12].max(0) as usize,
                }),
                3 => Some(Action::Dismiss),
                _ => None,
            };
            let hp = std::array::from_fn(|i| if a[20 + i] < 0 { None } else { Some(a[20 + i]) });
            let input = Input {
                tick: a[0].max(0) as usize,
                owner: &owner,
                fighters: &fighters,
                wall: &wall,
                captures: &captures,
                piece_hp: hp,
                controlled: a[9] != 0,
                automatic: a[10] != 0,
                action,
            };
            let commands = core.step(&input);
            Some(core.json(&commands))
        })()
        .unwrap_or_else(|| "{\"error\":\"Invalid Magnus frame or expired instance\"}".into());
        s.output = result.into_bytes();
        s.output.as_ptr()
    })
}

/// Lab-only setup. It modifies the local practice instance, never game files.
#[no_mangle]
pub extern "C" fn magnus_piece(handle: u32, piece: u32, x: i32, y: i32, hp: i32) -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let Some(c) = s.cores.get_mut(handle as usize).and_then(Option::as_mut) else {
            return 0;
        };
        let Some(p) = c.pieces.get_mut(piece as usize) else {
            return 0;
        };
        let cell = Cell::new(x.clamp(-120, 120) as i8, y.clamp(-120, 120) as i8);
        if !cell.valid() {
            return 0;
        }
        p.cell = cell;
        p.hp = (hp as i64).clamp(0, p.max_hp);
        1
    })
}
#[no_mangle]
pub extern "C" fn magnus_challenge(handle: u32, id: u32) -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let Some(c) = s.cores.get_mut(handle as usize).and_then(Option::as_mut) else {
            return 0;
        };
        if id >= 6 {
            return 0;
        }
        c.challenge = Some(Challenge::new(id as usize));
        1
    })
}

#[no_mangle]
pub extern "C" fn magnus_fixture(handle: u32, id: u32) -> *const u8 {
    use std::fmt::Write;
    SESSION.with(|s|{let mut s=s.borrow_mut();
        if id>=6||s.cores.get(handle as usize).is_none(){s.output=b"{\"error\":\"Unknown fixture\"}".to_vec();return s.output.as_ptr()}
        let f=magnus_fixtures::historical(id as usize);let mut j=format!("{{\"state\":{},\"fighters\":[",f.core.json(&[]));
        for(i,e)in f.fighters.iter().enumerate(){if i>0{j.push(',')}let _=write!(j,"{{\"id\":{},\"team\":{},\"x\":{},\"y\":{},\"hp\":{},\"maxHp\":{},\"attack\":{},\"ap\":{},\"champion\":true,\"visible\":true,\"targetable\":true,\"tags\":0}}",e.id,e.team,e.x,e.y,e.hp,e.max_hp,e.attack,e.ap);}
        let _=write!(j,"],\"walls\":{:?},\"moves\":[",f.walls);
        for(i,(t,p,c))in f.moves.iter().enumerate(){if i>0{j.push(',')}let _=write!(j,"[{t},{p},{},{}]",c.x,c.y);}
        j.push_str("],\"captures\":[");for(i,(t,p))in f.captures.iter().enumerate(){if i>0{j.push(',')}let _=write!(j,"[{t},{p}]");}
        j.push_str("],\"relocate\":[");for(i,(t,id,c))in f.relocate.iter().enumerate(){if i>0{j.push(',')}let _=write!(j,"[{t},{id},{},{}]",c.x,c.y);}
        let _=write!(j,"],\"end\":{}}}",f.end);
        s.cores[handle as usize]=Some(f.core);s.output=j.into_bytes();s.output.as_ptr()
    })
}
