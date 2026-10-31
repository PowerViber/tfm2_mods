//! The Windows side of the observe-only bridge: reading the live image, placing the hooks with every other thread
//! of the process suspended, and backtraces for the recorder.
//!
//! Placing hooks while the game runs: the thread ids are listed (allocating) BEFORE any thread is suspended, because
//! a suspended thread may hold the heap lock; while threads are suspended nothing here allocates. If any suspended
//! thread's instruction pointer is inside a patch span, everyone is resumed and it is retried (up to 50 times, 1 ms
//! apart) or abandoned. Each span's live bytes are checked again under suspension, then written with
//! VirtualProtect → write → restore protection → FlushInstructionCache.
use super::{detour, hex, identity, recorder, TARGETS};
use std::ffi::c_void;

type Handle = *mut c_void;

#[repr(C)]
struct ThreadEntry32 {
    size: u32,
    usage: u32,
    thread_id: u32,
    owner_process_id: u32,
    base_priority: i32,
    delta_priority: i32,
    flags: u32,
}

/// CONTEXT (x64) is 1232 bytes and 16-byte aligned; only ContextFlags (+0x30) and Rip (+0xF8) are used.
#[repr(C, align(16))]
struct Context([u8; 1232]);
const CONTEXT_CONTROL: u32 = 0x0010_0001;

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetCurrentProcess() -> Handle;
    fn GetCurrentProcessId() -> u32;
    fn GetCurrentThreadId() -> u32;
    fn VirtualAlloc(address: *mut c_void, size: usize, kind: u32, protect: u32) -> *mut c_void;
    fn VirtualProtect(address: *mut c_void, size: usize, protect: u32, old: *mut u32) -> i32;
    fn FlushInstructionCache(process: Handle, address: *const c_void, size: usize) -> i32;
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> Handle;
    fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry32) -> i32;
    fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry32) -> i32;
    fn OpenThread(access: u32, inherit: i32, id: u32) -> Handle;
    fn SuspendThread(thread: Handle) -> u32;
    fn ResumeThread(thread: Handle) -> u32;
    fn GetThreadContext(thread: Handle, context: *mut Context) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
    fn Sleep(ms: u32);
    fn RtlAddFunctionTable(table: *const RuntimeFunction, count: u32, base: u64) -> u8;
    fn RtlCaptureStackBackTrace(skip: u32, count: u32, frames: *mut *mut c_void, hash: *mut u32) -> u16;
}

#[repr(C)]
struct RuntimeFunction {
    begin: u32,
    end: u32,
    unwind: u32,
}
/// Where the region keeps its unwind table (RUNTIME_FUNCTION per hook) and the shared UNWIND_INFO.
const TABLE_AT: usize = 0xF000;
const UNWIND_AT: usize = 0xF800;

const MEM_COMMIT_RESERVE: u32 = 0x3000;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;
const TH32CS_SNAPTHREAD: u32 = 0x4;
const THREAD_SUSPEND_GET_CONTEXT: u32 = 0x0002 | 0x0008;
const INVALID: Handle = -1isize as Handle;

pub fn backtrace(frames: &mut [u64]) -> usize {
    let mut raw = [std::ptr::null_mut::<c_void>(); super::trace::FRAMES];
    let want = frames.len().min(raw.len()) as u32;
    // skip this function and the recorder: the first frame is the stub's caller-of-record chain
    let n = unsafe { RtlCaptureStackBackTrace(2, want, raw.as_mut_ptr(), std::ptr::null_mut()) } as usize;
    for (f, r) in frames.iter_mut().zip(&raw[..n]) {
        *f = *r as u64;
    }
    n
}

/// Executable memory within ±1.5 GB of the image, so relocated RIP-relative operands can still reach their targets.
fn near(base: u64, size: usize) -> Option<u64> {
    const STEP: u64 = 0x1_0000;
    for k in 1..(0x6000_0000 / STEP) {
        for at in [base.checked_sub(k * STEP), base.checked_add(identity::SIZE_OF_IMAGE as u64 + k * STEP)] {
            let Some(at) = at else { continue };
            let p = unsafe { VirtualAlloc(at as *mut c_void, size, MEM_COMMIT_RESERVE, PAGE_EXECUTE_READWRITE) };
            if !p.is_null() {
                return Some(p as u64);
            }
        }
    }
    None
}

struct Suspended {
    handles: Vec<Handle>,
}

impl Suspended {
    /// Suspend every other thread of the process; `None` (everything resumed) if one is inside a span.
    fn all_except_me(spans: &[(u64, u64)]) -> Result<Option<Self>, String> {
        let me = unsafe { GetCurrentThreadId() };
        let pid = unsafe { GetCurrentProcessId() };
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snap == INVALID {
            return Err("thread snapshot failed".into());
        }
        let mut ids = Vec::with_capacity(256);
        let mut e = ThreadEntry32 { size: std::mem::size_of::<ThreadEntry32>() as u32, usage: 0, thread_id: 0, owner_process_id: 0, base_priority: 0, delta_priority: 0, flags: 0 };
        let mut ok = unsafe { Thread32First(snap, &mut e) } != 0;
        while ok {
            if e.owner_process_id == pid && e.thread_id != me {
                ids.push(e.thread_id);
            }
            ok = unsafe { Thread32Next(snap, &mut e) } != 0;
        }
        unsafe { CloseHandle(snap) };
        // from here until resume: no allocation
        let mut s = Suspended { handles: Vec::with_capacity(ids.len()) };
        let mut ctx = Context([0; 1232]);
        for id in ids {
            let h = unsafe { OpenThread(THREAD_SUSPEND_GET_CONTEXT, 0, id) };
            if h.is_null() {
                continue; // the thread already ended
            }
            if unsafe { SuspendThread(h) } == u32::MAX {
                unsafe { CloseHandle(h) };
                continue;
            }
            s.handles.push(h);
            ctx.0[0x30..0x34].copy_from_slice(&CONTEXT_CONTROL.to_le_bytes());
            if unsafe { GetThreadContext(h, &mut ctx) } == 0 {
                drop(s);
                return Err("a thread's context could not be read".into());
            }
            let rip = u64::from_le_bytes(ctx.0[0xF8..0x100].try_into().unwrap_or_default());
            if spans.iter().any(|(lo, hi)| rip >= *lo && rip < *hi) {
                drop(s);
                return Ok(None);
            }
        }
        Ok(Some(s))
    }
}

impl Drop for Suspended {
    fn drop(&mut self) {
        for h in self.handles.drain(..) {
            unsafe {
                ResumeThread(h);
                CloseHandle(h);
            }
        }
    }
}

/// Called with threads suspended: reports a failure by address only (no allocation).
fn write_code(at: u64, bytes: &[u8]) -> Result<(), u64> {
    let mut old = 0u32;
    if unsafe { VirtualProtect(at as *mut c_void, bytes.len(), PAGE_EXECUTE_READWRITE, &mut old) } == 0 {
        return Err(at);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), at as *mut u8, bytes.len());
        VirtualProtect(at as *mut c_void, bytes.len(), old, &mut old);
        FlushInstructionCache(GetCurrentProcess(), at as *const c_void, bytes.len());
    }
    Ok(())
}

/// Identity, plans, stubs and trampolines first (nothing written to the game yet); then every patch under one
/// suspension. Returns the image base.
pub fn install() -> Result<u64, String> {
    let base = unsafe { GetModuleHandleW(std::ptr::null()) } as u64;
    if base == 0 {
        return Err("no game module".into());
    }
    let page = unsafe { std::slice::from_raw_parts(base as *const u8, 0x1000) };
    identity::supported(identity::parse(page))?;
    let mut plans = Vec::new();
    for t in &TARGETS {
        let ip = base + t.rva;
        let body = unsafe { std::slice::from_raw_parts(ip as *const u8, (t.end - t.rva) as usize) };
        let plan = detour::plan(ip, body, &hex(t.entry)).map_err(|e| format!("{}: {e}", t.name))?;
        plans.push((ip, plan));
    }
    let region = near(base, 0x1_0000).ok_or("no executable memory near the game image")?;
    let mut patches = Vec::new();
    for (n, (ip, plan)) in plans.iter().enumerate() {
        let slot = region + (n * detour::SLOT) as u64;
        let (stub_at, tramp_at) = (slot + detour::STUB_AT as u64, slot + detour::TRAMPOLINE_AT as u64);
        let tramp = detour::trampoline(*ip, plan, tramp_at).map_err(|e| format!("{}: {e}", TARGETS[n].name))?;
        let stub = detour::stub(n as u64, recorder as *const () as usize as u64, tramp_at);
        if tramp.len() > detour::SLOT - detour::TRAMPOLINE_AT || stub.len() > detour::TRAMPOLINE_AT {
            return Err(format!("{}: relocated code does not fit its slot", TARGETS[n].name));
        }
        unsafe {
            std::ptr::copy_nonoverlapping(stub.as_ptr(), stub_at as *mut u8, stub.len());
            std::ptr::copy_nonoverlapping(tramp.as_ptr(), tramp_at as *mut u8, tramp.len());
        }
        patches.push((*ip, detour::patch(plan, stub_at), plan.original.clone()));
    }
    // Unwind data for every stub, so backtraces and unwinders step through them (detour::stub_unwind).
    let unwind = detour::stub_unwind();
    unsafe { std::ptr::copy_nonoverlapping(unwind.as_ptr(), (region + UNWIND_AT as u64) as *mut u8, unwind.len()) };
    let table = (region + TABLE_AT as u64) as *mut RuntimeFunction;
    for n in 0..patches.len() {
        let begin = (n * detour::SLOT + detour::STUB_AT) as u32;
        unsafe {
            table.add(n).write(RuntimeFunction { begin, end: begin + detour::TRAMPOLINE_AT as u32, unwind: UNWIND_AT as u32 });
        }
    }
    if unsafe { RtlAddFunctionTable(table, patches.len() as u32, region) } == 0 {
        return Err("the stubs' unwind table was refused".into());
    }
    unsafe { FlushInstructionCache(GetCurrentProcess(), region as *const c_void, 0x1_0000) };
    let spans: Vec<(u64, u64)> = patches.iter().map(|(ip, p, _)| (*ip, *ip + p.len() as u64)).collect();
    for _ in 0..50 {
        let Some(held) = Suspended::all_except_me(&spans)? else {
            unsafe { Sleep(1) };
            continue;
        };
        // another thread or mod changed the bytes since planning: refuse, touching nothing
        if let Some((ip, _, _)) = patches.iter().find(|(ip, _, original)| {
            let live = unsafe { std::slice::from_raw_parts(*ip as *const u8, original.len()) };
            live != &original[..]
        }) {
            drop(held);
            return Err(format!("bytes at {:#x} changed before patching", ip - base));
        }
        let mut done = 0;
        let mut failure = None;
        for (ip, patch, _) in &patches {
            match write_code(*ip, patch) {
                Ok(()) => done += 1,
                Err(e) => {
                    failure = Some(e);
                    break;
                }
            }
        }
        if let Some(e) = failure {
            for (ip, _, original) in &patches[..done] {
                let _ = write_code(*ip, original);
            }
            drop(held);
            return Err(format!("VirtualProtect failed at {:#x}; every hook placed so far was removed", e - base));
        }
        drop(held);
        return Ok(base);
    }
    Err("a game thread stayed inside a hooked function's entry; no hook placed".into())
}
