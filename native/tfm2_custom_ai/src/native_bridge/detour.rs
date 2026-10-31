//! Planning one observe-only hook: which instructions the entry jump overwrites, where they go, and the bytes of
//! the stub, the trampoline and the patch. Pure: nothing here touches memory, so it is tested on any platform.
//!
//! Rules (PR #59's bridge requirements):
//! - every overwritten instruction is decoded whole and re-encoded by iced-x86's BlockEncoder at its new address, so
//!   RIP-relative operands and relative calls / jumps / branches keep their targets. Anything the encoder can't place
//!   (a RIP-relative target out of ±2 GB of the trampoline, say) is refused, never hand-patched;
//! - the patch must lie inside the function, and no instruction of the function may branch into the middle of it;
//! - the bytes there must be exactly the ones recorded for this build before anything is written.
use iced_x86::{BlockEncoder, BlockEncoderOptions, Decoder, DecoderOptions, FlowControl, Instruction, InstructionBlock};

/// `jmp qword ptr [rip+0]` followed by the 8-byte target: an absolute jump that clobbers no register.
pub const JUMP_LEN: usize = 14;

pub fn abs_jump(target: u64) -> [u8; JUMP_LEN] {
    let mut j = [0u8; JUMP_LEN];
    j[..6].copy_from_slice(&[0xFF, 0x25, 0, 0, 0, 0]);
    j[6..].copy_from_slice(&target.to_le_bytes());
    j
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Bytes of the function the patch overwrites (whole instructions, at least JUMP_LEN).
    pub len: usize,
    pub original: Vec<u8>,
}

/// Decide the patch span for a function starting at `ip` whose whole body (`ip .. ip + body.len()`) is `body`.
/// `expected` is the build's recorded entry bytes; they must match before anything else is considered.
pub fn plan(ip: u64, body: &[u8], expected: &[u8]) -> Result<Plan, String> {
    if expected.len() < JUMP_LEN || body.len() < expected.len() || body[..expected.len()] != *expected {
        return Err("entry bytes differ from this build's record (another mod, or another build)".into());
    }
    let mut decoder = Decoder::with_ip(64, body, ip, DecoderOptions::NONE);
    let mut span = Vec::new();
    let mut len = 0usize;
    while len < JUMP_LEN {
        if !decoder.can_decode() {
            return Err("function is shorter than the entry jump".into());
        }
        let i = decoder.decode();
        if i.is_invalid() {
            return Err(format!("undecodable instruction at +{len:#x}"));
        }
        len += i.len();
        span.push(i);
    }
    if len > expected.len() {
        return Err(format!("patch span {len} bytes runs past the {} recorded bytes", expected.len()));
    }
    let (lo, hi) = (ip, ip + len as u64);
    // Nothing may land inside the span: every instruction of the function is checked, the span's own included.
    let mut all = Decoder::with_ip(64, body, ip, DecoderOptions::NONE);
    for i in all.iter() {
        if i.is_invalid() {
            continue;
        }
        if matches!(
            i.flow_control(),
            FlowControl::UnconditionalBranch | FlowControl::ConditionalBranch | FlowControl::Call | FlowControl::XbeginXabortXend
        ) {
            let t = i.near_branch_target();
            if t > lo && t < hi {
                return Err(format!("instruction at {:#x} branches into the patch span ({t:#x})", i.ip()));
            }
        }
    }
    Ok(Plan { len, original: body[..len].to_vec() })
}

/// The trampoline: the overwritten instructions relocated to `at`, then a jump back to `ip + len`.
pub fn trampoline(ip: u64, plan: &Plan, at: u64) -> Result<Vec<u8>, String> {
    let mut decoder = Decoder::with_ip(64, &plan.original, ip, DecoderOptions::NONE);
    let insts: Vec<Instruction> = decoder.iter().collect();
    let block = InstructionBlock::new(&insts, at);
    let out = BlockEncoder::encode(64, block, BlockEncoderOptions::NONE).map_err(|e| format!("cannot relocate: {e}"))?;
    let mut code = out.code_buffer;
    code.extend_from_slice(&abs_jump(ip + plan.len as u64));
    Ok(code)
}

/// The bytes written over the function's entry: the jump to the stub, then int3 padding (never executed: nothing
/// branches into the span).
pub fn patch(plan: &Plan, stub: u64) -> Vec<u8> {
    let mut p = abs_jump(stub).to_vec();
    p.resize(plan.len, 0xCC);
    p
}

/// The entry stub, at any address: saves rax, rcx, rdx, r8–r11 and xmm0–5, keeps the stack 16-byte aligned with 32
/// bytes of shadow space, calls `recorder(id, regs)` (Microsoft x64 convention, `extern "win64"`) where `regs`
/// points at the saved [r11, r10, r9, r8, rdx, rcx, rax, return address], restores everything, then jumps to the
/// trampoline. It never reads or changes the function's private arguments beyond passing their raw values on.
///
/// Stack: entry rsp ≡ 8 (mod 16) (the caller's return address is on top); 7 pushes → ≡ 0; `sub rsp, 0x80` keeps 0,
/// so the recorder is entered with rsp ≡ 8 like any callee.
pub fn stub(id: u64, recorder: u64, trampoline: u64) -> Vec<u8> {
    let mut s = vec![
        0x50, 0x51, 0x52, 0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53, // push rax, rcx, rdx, r8, r9, r10, r11
        0x48, 0x81, 0xEC, 0x80, 0x00, 0x00, 0x00, // sub rsp, 0x80
    ];
    for (n, modrm) in [0x44u8, 0x4C, 0x54, 0x5C, 0x64, 0x6C].into_iter().enumerate() {
        s.extend_from_slice(&[0xF3, 0x0F, 0x7F, modrm, 0x24, 0x20 + 16 * n as u8]); // movdqu [rsp+0x20+16n], xmmN
    }
    s.extend_from_slice(&[0x48, 0xB9]); // mov rcx, id
    s.extend_from_slice(&id.to_le_bytes());
    s.extend_from_slice(&[0x48, 0x8D, 0x94, 0x24, 0x80, 0x00, 0x00, 0x00]); // lea rdx, [rsp+0x80]
    s.extend_from_slice(&[0x48, 0xB8]); // mov rax, recorder
    s.extend_from_slice(&recorder.to_le_bytes());
    s.extend_from_slice(&[0xFF, 0xD0]); // call rax
    for (n, modrm) in [0x44u8, 0x4C, 0x54, 0x5C, 0x64, 0x6C].into_iter().enumerate() {
        s.extend_from_slice(&[0xF3, 0x0F, 0x6F, modrm, 0x24, 0x20 + 16 * n as u8]); // movdqu xmmN, [rsp+0x20+16n]
    }
    s.extend_from_slice(&[
        0x48, 0x81, 0xC4, 0x80, 0x00, 0x00, 0x00, // add rsp, 0x80
        0x41, 0x5B, 0x41, 0x5A, 0x41, 0x59, 0x41, 0x58, 0x5A, 0x59, 0x58, // pop r11, r10, r9, r8, rdx, rcx, rax
    ]);
    s.extend_from_slice(&abs_jump(trampoline));
    s
}

/// Windows x64 UNWIND_INFO for the stub's prologue, so a stack walk taken inside the recorder (its backtrace, or
/// any unwinder) steps through the stub to the hooked function's caller. Prologue: push rax (ends +1), rcx (+2),
/// rdx (+3), r8 (+5), r9 (+7), r10 (+9), r11 (+11), sub rsp, 0x80 (+18); codes are listed last-first.
pub fn stub_unwind() -> Vec<u8> {
    const PUSH: u8 = 0; // UWOP_PUSH_NONVOL
    const ALLOC_SMALL: u8 = 2; // size = info * 8 + 8
    let codes: [(u8, u8, u8); 8] =
        [(18, ALLOC_SMALL, 15), (11, PUSH, 11), (9, PUSH, 10), (7, PUSH, 9), (5, PUSH, 8), (3, PUSH, 2), (2, PUSH, 1), (1, PUSH, 0)];
    let mut u = vec![0x01, 18, codes.len() as u8, 0x00]; // version 1, no flags; prologue 18 bytes; no frame register
    for (offset, op, info) in codes {
        u.extend_from_slice(&[offset, op | info << 4]);
    }
    u
}
/// End of the stub's prologue (where the unwind codes above stop).
#[cfg_attr(not(test), allow(dead_code))]
pub const STUB_PROLOGUE: usize = 18;

/// Room for one hook's stub + trampoline (the trampoline is at most JUMP_LEN + 15 bytes of relocated code each
/// growing to a long form, plus the jump back).
pub const SLOT: usize = 512;
pub const STUB_AT: usize = 0;
pub const TRAMPOLINE_AT: usize = 256;

#[cfg(test)]
#[path = "detour_tests.rs"]
mod tests;
