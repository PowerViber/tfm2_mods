//! Hook planning, relocation and the entry stub. The relocation and rejection tests are pure; the execution tests
//! run real hooked machine code in an mmap'd buffer (Linux x86-64 only), calling it with the Microsoft x64
//! convention exactly as the game would.
use super::*;

const BASE: u64 = 0x1_4000_0000;

/// f1(a, b, c, d) = a + b + c + d + K, with K loaded RIP-relative from +0x40 (inside the patch span).
fn f1() -> Vec<u8> {
    let mut c = vec![0x48, 0x8D, 0x04, 0x11, 0x4C, 0x01, 0xC0, 0x4C, 0x01, 0xC8, 0x48, 0x8B, 0x15];
    c.extend_from_slice(&(0x40i32 - 17).to_le_bytes());
    c.extend_from_slice(&[0x48, 0x01, 0xD0, 0xC3]);
    c
}
/// f2(a, b) = b == 0 ? a : a + 100, through a `je rel32` and a `call rel32` to a helper at +18.
fn f2() -> Vec<u8> {
    let mut c = vec![0x48, 0x89, 0xC8, 0x48, 0x85, 0xD2, 0x0F, 0x84];
    c.extend_from_slice(&5i32.to_le_bytes());
    c.push(0xE8);
    c.extend_from_slice(&1i32.to_le_bytes());
    c.push(0xC3);
    c
}
const F2_HELPER: [u8; 5] = [0x48, 0x83, 0xC0, 0x64, 0xC3];
/// f3(a) = a + 1, entered by a `jmp rel32` over dead padding.
fn f3() -> Vec<u8> {
    let mut c = vec![0xE9];
    c.extend_from_slice(&9i32.to_le_bytes());
    c.extend_from_slice(&[0x90; 9]);
    c.extend_from_slice(&[0x48, 0x8D, 0x41, 0x01, 0xC3]);
    c
}
/// f4(a) = a + K through `lea rax, [rip+K]`.
fn f4() -> Vec<u8> {
    let mut c = vec![0x48, 0x8D, 0x05];
    c.extend_from_slice(&(0x40i32 - 7).to_le_bytes());
    c.extend_from_slice(&[0x48, 0x8B, 0x00, 0x48, 0x01, 0xC8, 0x90, 0xC3]);
    c
}
/// f5(a, b) = a + 4b in doubles (xmm0, xmm1).
fn f5() -> Vec<u8> {
    let mut c = [0xF2, 0x0F, 0x58, 0xC1].repeat(4);
    c.push(0xC3);
    c
}

#[test]
fn span_covers_whole_instructions_and_keeps_them() {
    let code = f1();
    let p = plan(BASE, &code, &code).unwrap();
    assert_eq!(p.len, 17, "lea + add + add + the 7-byte RIP-relative mov: never a split instruction");
    assert_eq!(p.original, code[..17].to_vec());
    let patch = patch(&p, 0xDEAD_BEEF);
    assert_eq!(&patch[..6], &[0xFF, 0x25, 0, 0, 0, 0]);
    assert_eq!(&patch[14..], &[0xCC; 3], "padding is int3: nothing branches into it");
}

#[test]
fn relocation_keeps_every_target() {
    let at = BASE + 0x8100;
    // f2: the je and the call still reach the original function's ret and helper
    let code = f2();
    let p = plan(BASE, &code, &code).unwrap();
    let t = trampoline(BASE, &p, at).unwrap();
    let insts: Vec<_> = Decoder::with_ip(64, &t, at, DecoderOptions::NONE).iter().collect();
    let targets: Vec<u64> = insts.iter().filter(|i| i.near_branch_target() != 0).map(|i| i.near_branch_target()).collect();
    assert_eq!(targets, vec![BASE + 17, BASE + 18], "je → original ret, call → helper");
    assert_eq!(&t[t.len() - 14..], &abs_jump(BASE + 17), "then back into the function");
    // f1: the RIP-relative load still reads the same constant
    let code = f1();
    let p = plan(BASE, &code, &code).unwrap();
    let t = trampoline(BASE, &p, at).unwrap();
    // (the closing `jmp [rip+0]` reads its own target qword: leave it out)
    let rel: Vec<u64> = Decoder::with_ip(64, &t[..t.len() - 14], at, DecoderOptions::NONE)
        .iter()
        .filter(|i| i.is_ip_rel_memory_operand())
        .map(|i| i.ip_rel_memory_address())
        .collect();
    assert_eq!(rel, vec![BASE + 0x40]);
}

#[test]
fn unsafe_entries_are_refused() {
    // a loop branching back into the span
    let mut back = vec![0x48, 0x89, 0xC8, 0x48, 0xFF, 0xC0, 0x48, 0x83, 0xF8, 0x0A, 0x7C, 0xF7, 0x90, 0x90, 0x90, 0xC3];
    assert!(plan(BASE, &back, &back).unwrap_err().contains("branches into the patch span"));
    // a later instruction of the function jumping into the middle of the span
    back = vec![0x48, 0x8D, 0x04, 0x11, 0x4C, 0x01, 0xC0, 0x4C, 0x01, 0xC8, 0x48, 0x01, 0xC8, 0x90, 0xC3, 0xEB, 0xF3];
    let entry = back[..16].to_vec();
    assert!(plan(BASE, &back, &entry).unwrap_err().contains("branches into the patch span"));
    // too short to hold the jump
    let short = vec![0x48, 0x01, 0xC8, 0xC3];
    assert!(plan(BASE, &short, &short).is_err());
    // bytes that aren't this build's
    let code = f1();
    let mut other = code.clone();
    other[2] ^= 1;
    assert!(plan(BASE, &code, &other).unwrap_err().contains("entry bytes differ"));
    // a RIP-relative operand the trampoline can't reach (> 2 GB away) is not hand-patched
    let p = plan(BASE, &code, &code).unwrap();
    assert!(trampoline(BASE, &p, BASE + 0x1_0000_0000).unwrap_err().contains("cannot relocate"));
}

#[test]
fn the_stub_is_whole_instructions() {
    let s = stub(3, 0x1111_2222_3333_4444, 0x5555_6666_7777_8888);
    // (the last 8 bytes are the jump's target, not code)
    let insts: Vec<_> = Decoder::with_ip(64, &s[..s.len() - 8], 0, DecoderOptions::NONE).iter().collect();
    assert!(insts.iter().all(|i| !i.is_invalid()));
    // 7 pushes, sub, 6 saves, mov id, lea, mov recorder, call, 6 loads, add, 7 pops, the jump
    assert_eq!(insts.len(), 7 + 1 + 6 + 1 + 1 + 1 + 1 + 6 + 1 + 7 + 1);
    assert_eq!(&s[s.len() - 8..], &0x5555_6666_7777_8888u64.to_le_bytes());
}

#[test]
fn the_unwind_codes_match_the_stub_prologue() {
    let s = stub(0, 0, 0);
    // replay the prologue: each code's offset is where its instruction ends
    let ends: Vec<usize> = Decoder::with_ip(64, &s[..STUB_PROLOGUE], 0, DecoderOptions::NONE).iter().map(|i| i.next_ip() as usize).collect();
    assert_eq!(ends, vec![1, 2, 3, 5, 7, 9, 11, 18]);
    let u = stub_unwind();
    assert_eq!(u.len(), 4 + 2 * 8);
    let offsets: Vec<usize> = u[4..].chunks(2).map(|c| c[0] as usize).collect();
    assert_eq!(offsets, ends.iter().rev().copied().collect::<Vec<_>>());
    // the stack the codes undo equals what the prologue pushed: 7 × 8 + 0x80
    let undone: u32 = u[4..].chunks(2).map(|c| if c[1] & 0xF == 2 { (c[1] >> 4) as u32 * 8 + 8 } else { 8 }).sum();
    assert_eq!(undone, 7 * 8 + 0x80);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod run {
    use super::*;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

    extern "C" {
        fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut c_void;
    }

    struct Region(*mut u8);
    unsafe impl Send for Region {}
    fn region() -> Region {
        // PROT_READ | PROT_WRITE | PROT_EXEC, MAP_PRIVATE | MAP_ANONYMOUS
        let p = unsafe { mmap(std::ptr::null_mut(), 0x1_0000, 7, 0x22, -1, 0) };
        assert!(!p.is_null() && p as isize != -1, "an executable test buffer");
        Region(p as *mut u8)
    }

    static SEEN: [[AtomicU64; 5]; 8] = [const { [const { AtomicU64::new(0) }; 5] }; 8];

    #[repr(align(16))]
    struct Aligned([u8; 16]);

    /// Records what the stub passed, proves the stack is 16-byte aligned (movaps faults otherwise) and clobbers every
    /// volatile register the stub must restore.
    extern "win64" fn probe(id: u64, regs: *const u64) {
        let regs = unsafe { &*(regs as *const [u64; 8]) };
        let a = Aligned([7; 16]);
        unsafe {
            std::arch::asm!(
                "movaps xmm0, [{p}]",
                "xorps xmm1, xmm1", "xorps xmm2, xmm2", "xorps xmm3, xmm3", "xorps xmm4, xmm4", "xorps xmm5, xmm5",
                "mov r8, -1", "mov r9, -1", "mov r10, -1", "mov r11, -1",
                p = in(reg) a.0.as_ptr(),
                out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _, out("xmm4") _, out("xmm5") _,
                out("r8") _, out("r9") _, out("r10") _, out("r11") _,
            );
        }
        let s = &SEEN[id as usize];
        for (slot, v) in s.iter().zip([regs[5], regs[4], regs[3], regs[2], regs[7]]) {
            slot.store(v, SeqCst);
        }
    }

    /// Lay `code` (and extras) at +0x100, a constant K at +0x140, then hook it with slot +0x8000.
    fn install(r: &Region, code: &[u8], extra: &[u8]) -> u64 {
        let ip = r.0 as u64 + 0x100;
        unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), ip as *mut u8, code.len());
            std::ptr::copy_nonoverlapping(extra.as_ptr(), (ip + code.len() as u64) as *mut u8, extra.len());
            std::ptr::copy_nonoverlapping(0x1000u64.to_le_bytes().as_ptr(), (ip + 0x40) as *mut u8, 8);
        }
        ip
    }
    fn hook(r: &Region, ip: u64, body: usize, id: u64) {
        let code = unsafe { std::slice::from_raw_parts(ip as *const u8, body) }.to_vec();
        let p = plan(ip, &code, &code).unwrap();
        let slot = r.0 as u64 + 0x8000;
        let tramp_at = slot + TRAMPOLINE_AT as u64;
        let t = trampoline(ip, &p, tramp_at).unwrap();
        let s = stub(id, probe as *const () as usize as u64, tramp_at);
        let patch = patch(&p, slot);
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), slot as *mut u8, s.len());
            std::ptr::copy_nonoverlapping(t.as_ptr(), tramp_at as *mut u8, t.len());
            std::ptr::copy_nonoverlapping(patch.as_ptr(), ip as *mut u8, patch.len());
        }
    }
    type F4 = extern "win64" fn(u64, u64, u64, u64) -> u64;

    #[test]
    fn hooked_integer_functions_return_the_same_and_report_their_arguments() {
        for (id, code, extra) in [(1u64, f1(), vec![]), (2, f2(), F2_HELPER.to_vec()), (3, f3(), vec![]), (4, f4(), vec![])] {
            let r = region();
            let ip = install(&r, &code, &extra);
            let f: F4 = unsafe { std::mem::transmute(ip as usize) };
            let cases = [(1, 2, 3, 4), (5, 0, 7, 9), (u64::MAX / 3, 11, 0, 1)];
            let before: Vec<u64> = cases.iter().map(|c| f(c.0, c.1, c.2, c.3)).collect();
            hook(&r, ip, code.len(), id);
            let after: Vec<u64> = cases.iter().map(|c| f(c.0, c.1, c.2, c.3)).collect();
            assert_eq!(before, after, "function {id}: observing changes nothing");
            let s = &SEEN[id as usize];
            assert_eq!([0, 1, 2, 3].map(|k| s[k].load(SeqCst)), [u64::MAX / 3, 11, 0, 1], "function {id}: rcx, rdx, r8, r9");
            assert_ne!(s[4].load(SeqCst), 0, "function {id}: a return address");
        }
        // the hand-checked values
        let r = region();
        let ip = install(&r, &f2(), &F2_HELPER);
        hook(&r, ip, f2().len(), 5);
        let f: F4 = unsafe { std::mem::transmute(ip as usize) };
        assert_eq!((f(7, 0, 0, 0), f(7, 1, 0, 0)), (7, 107));
    }

    #[test]
    fn float_arguments_survive_the_stub() {
        let r = region();
        let code = f5();
        let ip = install(&r, &code, &[]);
        let f: extern "win64" fn(f64, f64) -> f64 = unsafe { std::mem::transmute(ip as usize) };
        let before = f(1.5, 2.25);
        hook(&r, ip, code.len(), 6);
        assert_eq!(f(1.5, 2.25), before);
        assert_eq!(before, 1.5 + 4.0 * 2.25);
    }
}
